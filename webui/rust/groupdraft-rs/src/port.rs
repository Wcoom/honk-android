//! The Rust port of `apps/lite/src/groupDraft.ts`.

#![forbid(unsafe_code)]

use crate::js;
use crate::utf16;
use dae_core::port::groups::read_group_entries_raw;
use dae_core::port::js_str::{
    is_space_or_tab, js_slice, js_slice_from, js_trim, js_trim_end, line_start_u16,
};
use dae_core::port::text::{TextBlock, TokenKind, is_bare_name_bool, scan_config_raw, unquote_str};
use serde_json::{Value, json};
use std::collections::HashSet;

const EMPTY_GROUP_FILTER: &str = "name(regex: '^$')";

fn find_group_entry(content: &str, name: &str) -> Option<TextBlock> {
    scan_config_raw(content)
        .blocks
        .into_iter()
        .filter(|b| b.name == "group")
        .flat_map(|b| b.children)
        .find(|b| b.name == name)
}

fn append_entry(text: &str, section: &str, entry: &str) -> String {
    let block = scan_config_raw(text)
        .blocks
        .into_iter()
        .rfind(|b| b.name == section);
    let Some(block) = block else {
        return format!("{}\n\n{section} {{\n  {entry}\n}}\n", js_trim_end(text));
    };
    let units = utf16::units(text);
    let line = line_start_u16(&units, block.close);
    let at = if is_space_or_tab(&js_slice(&units, line, block.close)) {
        line
    } else {
        block.close
    };
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let sep = if at > 0 && units.get(at - 1) == Some(&(b'\n' as u16)) {
        ""
    } else {
        eol
    };
    format!(
        "{}{sep}  {entry}{eol}{}",
        js_slice(&units, 0, at),
        js_slice_from(&units, at)
    )
}

fn remove_entry(text: &str, from: usize, to: usize) -> String {
    let units = utf16::units(text);
    format!("{}{}", js_slice(&units, 0, from), js_slice_from(&units, to))
}

fn refers_to(text: &str, name: &str) -> bool {
    let units = utf16::units(text);
    scan_config_raw(text).tokens.iter().any(|t| {
        matches!(t.kind, TokenKind::Text | TokenKind::Quoted)
            && unquote_str(&js_slice(&units, t.from, t.to)) == name
    })
}

fn without_groups(text: &str) -> String {
    let mut units = utf16::units(text);
    let blocks: Vec<TextBlock> = scan_config_raw(text)
        .blocks
        .into_iter()
        .filter(|b| b.name == "group")
        .collect();
    for block in blocks.into_iter().rev() {
        let end = block.to.min(units.len());
        let start = block.from.min(end);
        units.drain(start..end);
    }
    String::from_utf16_lossy(&units)
}

fn same_members(a: &[String], b: &[String]) -> bool {
    a.join("\n") == b.join("\n")
}

/// Port of `groupDraftEntry` (`apps/lite/src/groupDraft.ts:7`).
pub fn group_draft_entry(content: String, name: String) -> Result<Value, String> {
    Ok(match find_group_entry(&content, &name) {
        Some(block) => block.to_value(),
        None => js::undefined(),
    })
}

/// Port of `changeGroupText` (`apps/lite/src/groupDraft.ts:14`).
pub fn change_group_text(
    content: String,
    others: Vec<String>,
    original: String,
    name: String,
    body: String,
    duplicate: bool,
) -> Result<Value, String> {
    let entry = find_group_entry(&content, &original);
    let mut entries = read_group_entries_raw(&content);
    for other in &others {
        entries.extend(read_group_entries_raw(other));
    }
    let Some(entry) = entry else {
        return Err("incomplete".to_string());
    };
    if entries.iter().filter(|e| e.name == original).count() != 1 {
        return Err("incomplete".to_string());
    }
    let body_u = utf16::units(&body);
    let has_brace = scan_config_raw(&body).tokens.iter().any(|t| {
        t.kind == TokenKind::Symbol && matches!(js_slice(&body_u, t.from, t.to).as_str(), "{" | "}")
    });
    if has_brace {
        return Err("invalidField".to_string());
    }
    if duplicate {
        let next = js_trim(&name);
        let taken: HashSet<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        if !is_bare_name_bool(next) || taken.contains(next) {
            return Err("invalidField".to_string());
        }
        return Ok(Value::String(append_entry(
            &content,
            "group",
            &format!("{next} {{{body}}}"),
        )));
    }
    if name != original {
        return Err("invalidField".to_string());
    }
    let units = utf16::units(&content);
    Ok(Value::String(format!(
        "{}{body}{}",
        js_slice(&units, 0, entry.open + 1),
        js_slice_from(&units, entry.close)
    )))
}

/// Port of `deleteGroup` (`apps/lite/src/groupDraft.ts:34`).
pub fn delete_group(content: String, others: Vec<String>, name: String) -> Result<Value, String> {
    let Some(block) = find_group_entry(&content, &name) else {
        return Err("conflict".to_string());
    };
    let rest = remove_entry(&content, block.from, block.to);
    if others.iter().any(|text| refers_to(text, &name)) || refers_to(&rest, &name) {
        return Err("referenced".to_string());
    }
    Ok(Value::String(rest))
}

/// Port of `renameGroup` (`apps/lite/src/groupDraft.ts:42`).
pub fn rename_group(
    content: String,
    others: Vec<String>,
    original: String,
    name: String,
) -> Result<Value, String> {
    let Some(block) = find_group_entry(&content, &original) else {
        return Err("conflict".to_string());
    };
    let mut taken: HashSet<String> = read_group_entries_raw(&content)
        .into_iter()
        .map(|e| e.name)
        .filter(|n| n != &original)
        .collect();
    for other in &others {
        for entry in read_group_entries_raw(other) {
            if entry.name != original {
                taken.insert(entry.name);
            }
        }
    }
    if !is_bare_name_bool(&name) || taken.contains(&name) {
        return Err("invalidField".to_string());
    }
    if others.iter().any(|text| refers_to(text, &original)) {
        return Err("referenced".to_string());
    }
    let units = utf16::units(&content);
    let tokens = scan_config_raw(&content).tokens;
    let mut replacements: Vec<(usize, usize)> = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if !matches!(token.kind, TokenKind::Quoted | TokenKind::Text)
            || unquote_str(&js_slice(&units, token.from, token.to)) != original
        {
            continue;
        }
        let prev = |n: usize| -> String {
            i.checked_sub(n)
                .and_then(|idx| tokens.get(idx))
                .map(|t| js_slice(&units, t.from, t.to))
                .unwrap_or_default()
        };
        let p1 = prev(1);
        let p2 = prev(2);
        let allowed = token.from == block.from
            || p1 == "->"
            || (p1 == ":" && matches!(p2.as_str(), "fallback" | "route" | "download_detour"))
            || (p1 == "(" && p2 == "group");
        if allowed {
            replacements.push((token.from, token.to));
        } else {
            return Err("referenced".to_string());
        }
    }
    let name_u = utf16::units(&name);
    let mut out = units;
    for (from, to) in replacements.into_iter().rev() {
        let end = to.min(out.len());
        let start = from.min(end);
        out.splice(start..end, name_u.iter().copied());
    }
    Ok(Value::String(String::from_utf16_lossy(&out)))
}

/// Port of `groupChanges` (`apps/lite/src/groupDraft.ts:93`).
pub fn group_changes(before: String, after: String) -> Result<Vec<Value>, String> {
    let old = read_group_entries_raw(&before);
    let next = read_group_entries_raw(&after);
    let removed: Vec<_> = old
        .iter()
        .filter(|e| !next.iter().any(|item| item.name == e.name))
        .collect();
    let mut added: Vec<_> = next
        .iter()
        .filter(|e| !old.iter().any(|item| item.name == e.name))
        .collect();
    let mut changes: Vec<Value> = Vec::new();
    for entry in removed {
        if let Some(twin) = added.iter().position(|item| {
            same_members(&item.filters, &entry.filters) && item.policy == entry.policy
        }) {
            let renamed_to = added.remove(twin);
            changes.push(json!({
                "kind": "renamed",
                "before": entry.name,
                "after": renamed_to.name,
            }));
        } else {
            changes.push(json!({
                "kind": "removed",
                "name": entry.name,
            }));
        }
    }
    for entry in added {
        changes.push(json!({
            "kind": "added",
            "name": entry.name,
            "filters": entry.filters,
        }));
    }
    for entry in &next {
        let Some(was) = old.iter().find(|item| item.name == entry.name) else {
            continue;
        };
        if was.policy != entry.policy {
            changes.push(json!({
                "kind": "policy",
                "name": entry.name,
                "policy": entry.policy,
            }));
        }
        if !same_members(&was.filters, &entry.filters) {
            let added_f: Vec<&String> = entry
                .filters
                .iter()
                .filter(|f| !was.filters.contains(f))
                .collect();
            let removed_f: Vec<&String> = was
                .filters
                .iter()
                .filter(|f| !entry.filters.contains(f))
                .collect();
            changes.push(json!({
                "kind": "members",
                "name": entry.name,
                "added": added_f,
                "removed": removed_f,
            }));
        }
        if was.default != entry.default
            || was.final_ != entry.final_
            || was.interrupt != entry.interrupt
        {
            changes.push(json!({
                "kind": "changed",
                "name": entry.name,
            }));
        }
    }
    if without_groups(&before) != without_groups(&after) {
        changes.push(json!({ "kind": "other" }));
    }
    Ok(changes)
}

/// Port of `emptyGroups` (`apps/lite/src/groupDraft.ts:125`).
pub fn empty_groups(content: String) -> Result<Value, String> {
    let names: Vec<Value> = read_group_entries_raw(&content)
        .into_iter()
        .filter(|e| {
            !e.filters.is_empty() && e.filters.iter().all(|f| js_trim(f) == EMPTY_GROUP_FILTER)
        })
        .map(|e| Value::String(e.name))
        .collect();
    Ok(Value::Array(names))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "group {\n  proxy {\n    filter: subtag(harbor)\n    policy: min_moving_avg\n  }\n}\nrouting {\n  domain(geosite: cn) -> direct\n  fallback: proxy\n}\n";

    #[test]
    fn finds_and_misses_group_draft_entry() {
        let found = group_draft_entry(SAMPLE.to_string(), "proxy".to_string()).unwrap();
        assert_eq!(found["name"], "proxy");
        let missing = group_draft_entry(SAMPLE.to_string(), "absent".to_string()).unwrap();
        assert!(js::is_undefined(&missing));
    }

    #[test]
    fn edits_and_duplicates_group_text_with_validation() {
        let edited = change_group_text(
            SAMPLE.to_string(),
            vec![],
            "proxy".to_string(),
            "proxy".to_string(),
            "\n    filter: name(regex: '{ok}')\n    policy: random\n  ".to_string(),
            false,
        )
        .unwrap();
        assert!(edited.as_str().unwrap().contains("policy: random"));

        let dup = change_group_text(
            SAMPLE.to_string(),
            vec![],
            "proxy".to_string(),
            "  backup  ".to_string(),
            "\n    policy: fallback\n  ".to_string(),
            true,
        )
        .unwrap();
        assert!(dup.as_str().unwrap().contains("backup {"));

        assert_eq!(
            change_group_text(
                SAMPLE.to_string(),
                vec![],
                "proxy".to_string(),
                "proxy".to_string(),
                "policy: } bad".to_string(),
                false,
            ),
            Err("invalidField".to_string())
        );
        assert_eq!(
            change_group_text(
                SAMPLE.to_string(),
                vec!["group {\n  proxy {\n  }\n}\n".to_string()],
                "proxy".to_string(),
                "proxy".to_string(),
                "".to_string(),
                false,
            ),
            Err("incomplete".to_string())
        );
    }

    #[test]
    fn deletes_and_renames_groups_with_reference_guards() {
        let unused = "group {\n  spare {\n    policy: random\n  }\n}\n";
        let deleted = delete_group(unused.to_string(), vec![], "spare".to_string()).unwrap();
        assert!(!deleted.as_str().unwrap().contains("spare"));

        assert_eq!(
            delete_group(SAMPLE.to_string(), vec![], "proxy".to_string()),
            Err("referenced".to_string())
        );
        assert_eq!(
            delete_group(SAMPLE.to_string(), vec![], "missing".to_string()),
            Err("conflict".to_string())
        );

        let renamed = rename_group(
            SAMPLE.to_string(),
            vec![],
            "proxy".to_string(),
            "main".to_string(),
        )
        .unwrap();
        let renamed_s = renamed.as_str().unwrap();
        assert!(renamed_s.contains("main {"));
        assert!(renamed_s.contains("fallback: main"));

        assert_eq!(
            rename_group(
                SAMPLE.to_string(),
                vec!["routing {\n  fallback: proxy\n}\n".to_string()],
                "proxy".to_string(),
                "main".to_string(),
            ),
            Err("referenced".to_string())
        );
        assert_eq!(
            rename_group(
                SAMPLE.to_string(),
                vec![],
                "proxy".to_string(),
                "bad name".to_string(),
            ),
            Err("invalidField".to_string())
        );
    }

    #[test]
    fn detects_group_changes_and_empty_groups() {
        let renamed_cfg = rename_group(
            SAMPLE.to_string(),
            vec![],
            "proxy".to_string(),
            "main".to_string(),
        )
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
        let changes = group_changes(SAMPLE.to_string(), renamed_cfg).unwrap();
        assert_eq!(
            changes,
            vec![
                json!({"kind": "renamed", "before": "proxy", "after": "main"}),
                json!({"kind": "other"}),
            ]
        );

        let with_empty = "group {\n  blank {\n    filter: name(regex: '^$')\n  }\n  ok {\n    filter: subtag(a)\n  }\n}\n";
        assert_eq!(
            empty_groups(with_empty.to_string()).unwrap(),
            json!(["blank"])
        );
    }
}
