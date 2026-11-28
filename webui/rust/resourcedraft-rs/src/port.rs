//! The Rust port of `apps/lite/src/resourceDraft.ts`.

#![forbid(unsafe_code)]

use crate::utf16;
use dae_core::port::js_str::{is_space_or_tab, js_trim, js_trim_end, line_start_u16};
use dae_core::port::nodes::is_node_link_bool;
use dae_core::port::subscriptions::complete_subscription_url_str;
use dae_core::port::text::{is_quotable_bool, quote_str, scan_config_raw};
use serde_json::{Value, json};

struct DraftEntry {
    name: String,
    value: String,
    from: usize,
    to: usize,
}

fn read_drafts(content: &str, kind: &str) -> Result<Vec<DraftEntry>, String> {
    let items = if kind == "node" {
        dae_core::port::read_node_entries(content.to_string())?
    } else {
        dae_core::port::read_subscription_entries(content.to_string())?
    };
    let (name_key, value_key) = if kind == "node" {
        ("name", "link")
    } else {
        ("tag", "url")
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let Some(o) = item.as_object() else {
            continue;
        };
        let name = o
            .get(name_key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let value = o
            .get(value_key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let from = o
            .get("from")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(0);
        let to = o
            .get("to")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(0);
        out.push(DraftEntry {
            name,
            value,
            from,
            to,
        });
    }
    Ok(out)
}

fn resource_referenced(contents: &[&str], kind: &str, name: &str) -> Result<bool, String> {
    for &content in contents {
        let hit = if kind == "node" {
            dae_core::port::node_referenced(content.to_string(), name.to_string())?
                == Value::Bool(true)
        } else {
            dae_core::port::groups_naming_tag(content.to_string(), name.to_string())?
                .as_array()
                .is_some_and(|a| !a.is_empty())
        };
        if hit {
            return Ok(true);
        }
    }
    Ok(false)
}

fn append_entry(text: &str, section: &str, entry: &str) -> String {
    let scan = scan_config_raw(text);
    let Some(block) = scan.blocks.iter().rfind(|b| b.name == section) else {
        return format!("{}\n\n{section} {{\n  {entry}\n}}\n", js_trim_end(text));
    };
    let units = utf16::units(text);
    let line = line_start_u16(&units, block.close);
    let at = if is_space_or_tab(&utf16::slice(&units, line, block.close)) {
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
        utf16::slice(&units, 0, at),
        utf16::slice(&units, at, units.len())
    )
}

fn remove_entry(text: &str, from: usize, to: usize) -> String {
    let units = utf16::units(text);
    format!(
        "{}{}",
        utf16::slice(&units, 0, from),
        utf16::slice(&units, to, units.len())
    )
}

/// Port of `draftResources` (`apps/lite/src/resourceDraft.ts:9`).
pub fn draft_resources(content: String, kind: String) -> Result<Value, String> {
    let entries = read_drafts(&content, &kind)?;
    Ok(Value::Array(
        entries
            .into_iter()
            .map(|e| json!({"name": e.name, "value": e.value, "from": e.from, "to": e.to}))
            .collect(),
    ))
}

/// Port of `changeResource` (`apps/lite/src/resourceDraft.ts:17`).
pub fn change_resource(
    content: String,
    others: Vec<String>,
    kind: String,
    action: String,
    original: String,
    name: String,
    value: String,
) -> Result<Value, String> {
    let own = read_drafts(&content, &kind)?;
    let Some(entry) = own.into_iter().find(|e| e.name == original) else {
        return Err("incomplete".into());
    };
    let mut all_texts: Vec<&str> = Vec::with_capacity(1 + others.len());
    all_texts.push(&content);
    all_texts.extend(others.iter().map(String::as_str));
    let mut total_matches = 0usize;
    for &text in &all_texts {
        total_matches += read_drafts(text, &kind)?
            .iter()
            .filter(|e| e.name == original)
            .count();
    }
    if total_matches != 1 {
        return Err("incomplete".into());
    }
    if action == "delete" {
        if resource_referenced(&all_texts, &kind, &original)? {
            return Err("referenced".into());
        }
        return Ok(Value::String(remove_entry(&content, entry.from, entry.to)));
    }
    let next = js_trim(&name);
    let valid_val = if kind == "node" {
        is_node_link_bool(&value)
    } else {
        dae_core::port::is_subscription_url(value.clone())? == Value::Bool(true)
    };
    if next.is_empty() || !is_quotable_bool(next) || !is_quotable_bool(&value) || !valid_val {
        return Err("invalidField".into());
    }
    if action == "duplicate" {
        for &text in &all_texts {
            if read_drafts(text, &kind)?.iter().any(|e| e.name == next) {
                return Err("invalidField".into());
            }
        }
        if kind == "node" {
            let line = format!("{}: {}", quote_str(next)?, quote_str(js_trim(&value))?);
            return Ok(Value::String(append_entry(&content, "node", &line)));
        }
        let units = utf16::units(&content);
        let snippet = utf16::slice(&units, entry.from, entry.to);
        let copy = dae_core::port::write_subscription_entry(
            format!("subscription {{\n{snippet}\n}}"),
            original,
            json!({"tag": next, "url": complete_subscription_url_str(&value)}),
        )?;
        let dup_list = read_drafts(&copy, &kind)?;
        let Some(dup) = dup_list.first() else {
            return Err("incomplete".into());
        };
        let copy_units = utf16::units(&copy);
        let dup_text = utf16::slice(&copy_units, dup.from, dup.to);
        return Ok(Value::String(append_entry(
            &content,
            "subscription",
            &dup_text,
        )));
    }
    if next != original {
        return Err("invalidField".into());
    }
    let updated = if kind == "node" {
        let orig_node = dae_core::port::read_node_entries(content.clone())?
            .into_iter()
            .find(|e| e.get("name").and_then(Value::as_str) == Some(original.as_str()))
            .ok_or_else(|| "incomplete".to_string())?;
        dae_core::port::write_node_entry(
            content,
            orig_node,
            json!({"name": next, "link": js_trim(&value)}),
        )?
    } else {
        dae_core::port::write_subscription_entry(
            content,
            original,
            json!({"url": complete_subscription_url_str(&value)}),
        )?
    };
    Ok(Value::String(updated))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "node {\n  a: 'vless://id@host:443'\n  b: 'vless://id@b:443'\n}\nsubscription {\n  sub1: 'https://example.net/sub'\n}\ngroup {\n  p {\n    filter: name(a)\n  }\n}\n";

    #[test]
    fn drafts_and_delete_referenced() {
        let nodes = draft_resources(SAMPLE.into(), "node".into()).expect("nodes");
        assert_eq!(nodes.as_array().map(Vec::len), Some(2));
        let err = change_resource(
            SAMPLE.into(),
            vec![],
            "node".into(),
            "delete".into(),
            "a".into(),
            "a".into(),
            String::new(),
        );
        assert_eq!(err, Err("referenced".into()));
        let deleted = change_resource(
            SAMPLE.into(),
            vec![],
            "node".into(),
            "delete".into(),
            "b".into(),
            "b".into(),
            String::new(),
        )
        .expect("delete b");
        assert!(!deleted.as_str().unwrap_or("").contains("vless://id@b:443"));
    }

    #[test]
    fn duplicate_and_update_resources() {
        let dup = change_resource(
            SAMPLE.into(),
            vec![],
            "subscription".into(),
            "duplicate".into(),
            "sub1".into(),
            "sub2".into(),
            "example.net/two".into(),
        )
        .expect("dup sub");
        assert!(
            dup.as_str()
                .unwrap_or("")
                .contains("sub2: 'https://example.net/two'")
        );
        let edit = change_resource(
            SAMPLE.into(),
            vec![],
            "node".into(),
            "text".into(),
            "a".into(),
            "a".into(),
            "vless://new@host:443".into(),
        )
        .expect("edit node");
        assert!(edit.as_str().unwrap_or("").contains("vless://new@host:443"));
        let bad = change_resource(
            SAMPLE.into(),
            vec![],
            "node".into(),
            "text".into(),
            "a".into(),
            "renamed".into(),
            "vless://new@host:443".into(),
        );
        assert_eq!(bad, Err("invalidField".into()));
    }
}
