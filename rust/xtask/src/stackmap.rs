//! Pure Rust port of `stackmap.py`: computes reachable BPF subprograms and chained stack depths.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::process::Command;

#[derive(Default, Debug)]
struct FuncInfo {
    stack: u64,
    calls: BTreeSet<String>,
    n: usize,
}

fn parse_symbols(readelf_s: &str) -> HashMap<(String, u64), String> {
    let mut syms = HashMap::new();
    for line in readelf_s.lines() {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() >= 8
            && p[3] == "FUNC"
            && let Ok(addr) = u64::from_str_radix(p[1], 16)
        {
            syms.insert((p[6].to_string(), addr), p[7].to_string());
        }
    }
    syms
}

fn parse_section_indices(readelf_big_s: &str) -> HashMap<String, String> {
    let mut secidx = HashMap::new();
    for line in readelf_big_s.lines() {
        let trimmed = line.trim_start();
        let Some(after_open) = trimmed.strip_prefix('[') else {
            continue;
        };
        let Some((idx_raw, rest)) = after_open.split_once(']') else {
            continue;
        };
        let idx = idx_raw.trim();
        if idx.is_empty() || !idx.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        if let Some(name) = rest.split_whitespace().next() {
            secidx.insert(name.to_string(), idx.to_string());
        }
    }
    secidx
}

fn extract_r10_offsets(ins: &str) -> Vec<u64> {
    let mut offsets = Vec::new();
    let needle = "r10 - 0x";
    let mut rest = ins;
    while let Some(pos) = rest.find(needle) {
        let after = &rest[pos + needle.len()..];
        let hex_len = after.bytes().take_while(|b| b.is_ascii_hexdigit()).count();
        if hex_len > 0
            && let Ok(v) = u64::from_str_radix(&after[..hex_len], 16)
        {
            offsets.push(v);
        }
        rest = &after[hex_len..];
    }
    offsets
}

fn parse_signed_hex(s: &str) -> Option<i64> {
    if let Some(rest) = s.strip_prefix("-0x") {
        i64::from_str_radix(rest, 16).ok().map(|v| -v)
    } else if let Some(rest) = s.strip_prefix("0x") {
        i64::from_str_radix(rest, 16).ok()
    } else {
        None
    }
}

fn parse_call_imm(ins: &str) -> Option<i64> {
    let rest = ins.strip_prefix("call ")?;
    if rest.contains(char::is_whitespace) {
        return None;
    }
    parse_signed_hex(rest)
}

fn parse_disassembly(
    objdump: &str,
    syms: &HashMap<(String, u64), String>,
    text_idx: &str,
) -> HashMap<String, FuncInfo> {
    let mut funcs: HashMap<String, FuncInfo> = HashMap::new();
    let mut cur: Option<String> = None;
    let mut cursec: Option<String> = None;
    let mut pending: Option<(i64, i64)> = None;

    for line in objdump.lines() {
        if let Some(rest) = line.strip_prefix("Disassembly of section ")
            && let Some(sec) = rest.strip_suffix(':')
        {
            cursec = Some(sec.to_string());
            continue;
        }
        if let Some((hex_part, rest)) = line.split_once(" <")
            && !hex_part.is_empty()
            && hex_part.bytes().all(|b| b.is_ascii_hexdigit())
            && let Some(name) = rest.strip_suffix(">:")
        {
            cur = Some(name.to_string());
            funcs.insert(name.to_string(), FuncInfo::default());
            continue;
        }
        let Some(ref cur_fn) = cur else {
            continue;
        };

        if !line.contains("R_BPF")
            && (line.starts_with(' ') || line.starts_with('\t'))
            && let Some((pc_raw, ins_raw)) = line.trim_start().split_once(':')
            && !pc_raw.is_empty()
            && pc_raw.bytes().all(|b| b.is_ascii_digit())
            && let Ok(pc) = pc_raw.parse::<i64>()
        {
            let ins = ins_raw.trim_start();
            if let Some(info) = funcs.get_mut(cur_fn) {
                info.n += 1;
                for off in extract_r10_offsets(ins) {
                    if off > info.stack {
                        info.stack = off;
                    }
                }
            }
            pending = None;
            if let Some(imm) = parse_call_imm(ins) {
                pending = Some((pc, imm));
                if cursec.as_deref() == Some(".text")
                    && let Some(target_addr) = (pc + 1 + imm).checked_mul(8)
                    && let Ok(addr_u64) = u64::try_from(target_addr)
                    && let Some(tgt) = syms.get(&(text_idx.to_string(), addr_u64))
                    && let Some(info) = funcs.get_mut(cur_fn)
                {
                    info.calls.insert(tgt.clone());
                }
            }
            continue;
        }

        if let Some(pos) = line.find("R_BPF_64_32")
            && let Some((_pc, imm)) = pending
        {
            let after = &line[pos + "R_BPF_64_32".len()..];
            if let Some(rel_target) = after.split_whitespace().next() {
                if rel_target == ".text" && cursec.as_deref() != Some(".text") {
                    if let Some(target_addr) = (imm + 1).checked_mul(8)
                        && let Ok(addr_u64) = u64::try_from(target_addr)
                        && let Some(tgt) = syms.get(&(text_idx.to_string(), addr_u64))
                        && let Some(info) = funcs.get_mut(cur_fn)
                    {
                        info.calls.insert(tgt.clone());
                    }
                } else if (funcs.contains_key(rel_target) || rel_target != ".text")
                    && let Some(info) = funcs.get_mut(cur_fn)
                {
                    info.calls.insert(rel_target.to_string());
                }
            }
        }
    }

    funcs
}

pub fn short_name(n: &str) -> String {
    if !n.starts_with("_R") {
        return n.to_string();
    }
    let bytes = n.as_bytes();
    let mut parts = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                parts.push(&n[start..i]);
            }
        } else {
            i += 1;
        }
    }
    if parts.is_empty() {
        n.to_string()
    } else {
        let start = parts.len().saturating_sub(2);
        parts[start..].join("::")
    }
}

fn walk(
    funcs: &HashMap<String, FuncInfo>,
    f: &str,
    depth: usize,
    acc: u64,
    seen: &HashSet<String>,
    out: &mut String,
) {
    let info = funcs.get(f);
    let st = info.map_or(0, |i| i.stack);
    let n = info.map_or(0, |i| i.n);
    let rounded = st.div_ceil(32) * 32;
    let indent = "  ".repeat(depth);
    out.push_str(&format!(
        "{indent}{}  stack={st} insns={n}  chain={}\n",
        short_name(f),
        acc + rounded
    ));
    if seen.contains(f) {
        return;
    }
    let mut next_seen = seen.clone();
    next_seen.insert(f.to_string());
    if let Some(info) = info {
        for c in &info.calls {
            if funcs.contains_key(c) {
                walk(funcs, c, depth + 1, acc + rounded, &next_seen, out);
            }
        }
    }
}

pub fn render_stackmap(
    readelf_s: &str,
    readelf_big_s: &str,
    objdump: &str,
    entry: &str,
) -> Result<String, String> {
    let syms = parse_symbols(readelf_s);
    let secidx = parse_section_indices(readelf_big_s);
    let text_idx = secidx
        .get(".text")
        .ok_or_else(|| "missing .text section in ELF".to_string())?;
    let funcs = parse_disassembly(objdump, &syms, text_idx);
    let mut out = String::new();
    walk(&funcs, entry, 0, 0, &HashSet::new(), &mut out);
    Ok(out)
}

pub fn run_stackmap(obj: &Path, entry: &str) -> Result<(), String> {
    let readelf_s = Command::new("llvm-readelf-18")
        .args(["-sW"])
        .arg(obj)
        .output()
        .map_err(|e| format!("failed to run llvm-readelf-18 -sW: {e}"))?;
    let readelf_big_s = Command::new("llvm-readelf-18")
        .args(["-SW"])
        .arg(obj)
        .output()
        .map_err(|e| format!("failed to run llvm-readelf-18 -SW: {e}"))?;
    let objdump = Command::new("llvm-objdump-18")
        .args(["-d", "--no-show-raw-insn", "-r"])
        .arg(obj)
        .output()
        .map_err(|e| format!("failed to run llvm-objdump-18: {e}"))?;

    let rendered = render_stackmap(
        &String::from_utf8_lossy(&readelf_s.stdout),
        &String::from_utf8_lossy(&readelf_big_s.stdout),
        &String::from_utf8_lossy(&objdump.stdout),
        entry,
    )?;
    print!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortens_rust_v0_mangled_names_identically_to_stackmap_py() {
        assert_eq!(short_name("wan_egress_l3"), "wan_egress_l3");
        // Matches `re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]*)", n)` in stackmap.py
        assert_eq!(
            short_name("_RNvMs1_NtCs123_9honk_ebpf6egress12handle_frame"),
            "_NtCs123_9honk_ebpf6egress12handle_frame"
        );
        assert_eq!(
            short_name("_RNvNtCs4abc_9honk.ebpf6egress.12handle_frame"),
            "egress::handle_frame"
        );
    }

    #[test]
    fn parses_r10_offsets_and_signed_call_immediates() {
        assert_eq!(
            extract_r10_offsets("*(u64 *)(r10 - 0x18) = r1; r2 = r10 - 0x40"),
            vec![0x18, 0x40]
        );
        assert_eq!(parse_call_imm("call 0x12"), Some(0x12));
        assert_eq!(parse_call_imm("call -0x4"), Some(-4));
        assert_eq!(parse_call_imm("call 1"), None);
    }
}
