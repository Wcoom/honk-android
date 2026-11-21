#!/usr/bin/env python3
"""stackmap.py OBJ ENTRY —— 列出某入口程序可达的子程序及各自栈深（按 r10 最小偏移估算）
用法: stackmap.py honk-ebpf wan_egress_l2
"""
import re, subprocess, sys
obj, entry = sys.argv[1], sys.argv[2]
# symbol table: (section, addr) -> name
syms = {}
for line in subprocess.run(["llvm-readelf-18", "-sW", obj], capture_output=True, text=True).stdout.splitlines():
    p = line.split()
    if len(p) >= 8 and p[3] == "FUNC":
        syms[(p[6], int(p[1], 16))] = p[7]
secidx = {}
for line in subprocess.run(["llvm-readelf-18", "-SW", obj], capture_output=True, text=True).stdout.splitlines():
    m = re.match(r"\s*\[\s*(\d+)\]\s+(\S+)", line)
    if m: secidx[m.group(2)] = m.group(1)
text_idx = secidx[".text"]
out = subprocess.run(["llvm-objdump-18", "-d", "--no-show-raw-insn", "-r", obj],
                     capture_output=True, text=True).stdout
funcs = {}
cur = None; cursec = None; pending = None
for line in out.splitlines():
    m = re.match(r"^Disassembly of section (\S+):", line)
    if m: cursec = m.group(1); continue
    m = re.match(r"^([0-9a-f]+) <(.+)>:", line)
    if m:
        cur = m.group(2); funcs[cur] = {"stack": 0, "calls": set(), "n": 0}; continue
    if cur is None: continue
    m = re.match(r"^\s+(\d+):\s+(.*)$", line)
    if m and "R_BPF" not in line:
        pc = int(m.group(1)); ins = m.group(2)
        funcs[cur]["n"] += 1
        for off in re.findall(r"r10 - 0x([0-9a-f]+)", ins):
            funcs[cur]["stack"] = max(funcs[cur]["stack"], int(off, 16))
        mc = re.match(r"call (-?0x[0-9a-f]+)$", ins)
        pending = None
        if mc:
            imm = int(mc.group(1), 16)
            pending = (pc, imm)
            if cursec == ".text":
                tgt = syms.get((text_idx, (pc + 1 + imm) * 8))
                if tgt: funcs[cur]["calls"].add(tgt)
        continue
    mr = re.search(r"R_BPF_64_32\s+(\S+)", line)
    if mr and pending:
        pc, imm = pending
        if mr.group(1) == ".text" and cursec != ".text":
            tgt = syms.get((text_idx, (imm + 1) * 8))
            if tgt: funcs[cur]["calls"].add(tgt)
        elif mr.group(1) in funcs or mr.group(1) not in (".text",):
            funcs[cur]["calls"].add(mr.group(1))
def short(n):
    if not n.startswith("_R"): return n
    m = re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]*)", n)
    return "::".join(m[-2:]) if m else n
def walk(f, depth, acc, seen):
    st = funcs.get(f, {}).get("stack", 0)
    r = ((st + 31) // 32) * 32
    print("  " * depth + f"{short(f)}  stack={st} insns={funcs.get(f,{}).get('n',0)}  chain={acc + r}")
    if f in seen: return
    for c in sorted(funcs.get(f, {}).get("calls", [])):
        if c in funcs: walk(c, depth + 1, acc + r, seen | {f})
walk(entry, 0, 0, set())
