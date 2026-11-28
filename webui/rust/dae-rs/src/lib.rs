//! JS bindings for `src/dae/`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;
pub mod utf16;

/// `serializeConditions`.
fn serialize_conditions_inner(rows: String) -> Result<Option<String>, String> {
    let rows = crate::js::from_json(&rows)?;
    let out = port::serialize_conditions(rows)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "serializeConditions")]
pub fn serialize_conditions(rows: String) -> Result<Option<String>, JsError> {
    serialize_conditions_inner(rows).map_err(|e| JsError::new(&e))
}

/// `parseConditions`.
fn parse_conditions_inner(text: String, kinds: Vec<String>) -> Result<String, String> {
    let out = port::parse_conditions(text, kinds)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "parseConditions")]
pub fn parse_conditions(text: String, kinds: Vec<String>) -> Result<String, JsError> {
    parse_conditions_inner(text, kinds).map_err(|e| JsError::new(&e))
}

/// `flagKey`.
fn flag_key_inner(name: String) -> Result<String, String> {
    let out = port::flag_key(name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "flagKey")]
pub fn flag_key(name: String) -> Result<String, JsError> {
    flag_key_inner(name).map_err(|e| JsError::new(&e))
}

/// `validFlagName`.
fn valid_flag_name_inner(name: String) -> Result<String, String> {
    let out = port::valid_flag_name(name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "validFlagName")]
pub fn valid_flag_name(name: String) -> Result<String, JsError> {
    valid_flag_name_inner(name).map_err(|e| JsError::new(&e))
}

/// `hasEmbeddedFlag`.
fn has_embedded_flag_inner(name: String) -> Result<String, String> {
    let out = port::has_embedded_flag(name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "hasEmbeddedFlag")]
pub fn has_embedded_flag(name: String) -> Result<String, JsError> {
    has_embedded_flag_inner(name).map_err(|e| JsError::new(&e))
}

/// `validFlagChoice`.
fn valid_flag_choice_inner(value: String) -> Result<bool, String> {
    let value = crate::js::from_json(&value)?;
    let out = port::valid_flag_choice(value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "validFlagChoice")]
pub fn valid_flag_choice(value: String) -> Result<bool, JsError> {
    valid_flag_choice_inner(value).map_err(|e| JsError::new(&e))
}

/// `flagChoices`.
fn flag_choices_inner(locale: String) -> Result<String, String> {
    let out = port::flag_choices(locale)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "flagChoices")]
pub fn flag_choices(locale: String) -> Result<String, JsError> {
    flag_choices_inner(locale).map_err(|e| JsError::new(&e))
}

/// `matchPreset`.
fn match_preset_inner(urls: String) -> Result<String, String> {
    let urls = crate::js::from_json(&urls)?;
    let out = port::match_preset(urls)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "matchPreset")]
pub fn match_preset(urls: String) -> Result<String, JsError> {
    match_preset_inner(urls).map_err(|e| JsError::new(&e))
}

/// `missingCategories`.
fn missing_categories_inner(preset: String, required: String) -> Result<String, String> {
    let preset = crate::js::from_json(&preset)?;
    let required = crate::js::from_json(&required)?;
    let out = port::missing_categories(preset, required)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "missingCategories")]
pub fn missing_categories(preset: String, required: String) -> Result<String, JsError> {
    missing_categories_inner(preset, required).map_err(|e| JsError::new(&e))
}

/// `geodataPreset`.
fn geodata_preset_inner(id: String) -> Result<String, String> {
    let out = port::geodata_preset(id)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "geodataPreset")]
pub fn geodata_preset(id: String) -> Result<String, JsError> {
    geodata_preset_inner(id).map_err(|e| JsError::new(&e))
}

/// `validGeodataUrl`.
fn valid_geodata_url_inner(url: String) -> Result<bool, String> {
    let out = port::valid_geodata_url(url)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "validGeodataUrl")]
pub fn valid_geodata_url(url: String) -> Result<bool, JsError> {
    valid_geodata_url_inner(url).map_err(|e| JsError::new(&e))
}

/// `conditionFamily`.
fn condition_family_inner(kind: String) -> Result<String, String> {
    let kind = crate::js::from_json(&kind)?;
    let out = port::condition_family(kind)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "conditionFamily")]
pub fn condition_family(kind: String) -> Result<String, JsError> {
    condition_family_inner(kind).map_err(|e| JsError::new(&e))
}

/// `newGroupCondition`.
fn new_group_condition_inner() -> Result<String, String> {
    let out = port::new_group_condition()?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "newGroupCondition")]
pub fn new_group_condition() -> Result<String, JsError> {
    new_group_condition_inner().map_err(|e| JsError::new(&e))
}

/// `conditionValues`.
fn condition_values_inner(values: Vec<String>) -> Result<String, String> {
    let out = port::condition_values(values)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "conditionValues")]
pub fn condition_values(values: Vec<String>) -> Result<String, JsError> {
    condition_values_inner(values).map_err(|e| JsError::new(&e))
}

/// `parseConditionValues`.
fn parse_condition_values_inner(text: String) -> Result<String, String> {
    let out = port::parse_condition_values(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "parseConditionValues")]
pub fn parse_condition_values(text: String) -> Result<String, JsError> {
    parse_condition_values_inner(text).map_err(|e| JsError::new(&e))
}

/// `parseGroupConditions`.
fn parse_group_conditions_inner(source: String) -> Result<String, String> {
    let out = port::parse_group_conditions(source)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "parseGroupConditions")]
pub fn parse_group_conditions(source: String) -> Result<String, JsError> {
    parse_group_conditions_inner(source).map_err(|e| JsError::new(&e))
}

/// `serializeGroupConditions`.
fn serialize_group_conditions_inner(rows: String) -> Result<Option<String>, String> {
    let rows = crate::js::from_json(&rows)?;
    let out = port::serialize_group_conditions(rows)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "serializeGroupConditions")]
pub fn serialize_group_conditions(rows: String) -> Result<Option<String>, JsError> {
    serialize_group_conditions_inner(rows).map_err(|e| JsError::new(&e))
}

/// `groupFilterDraft`.
fn group_filter_draft_inner(source: String, advanced: Option<bool>) -> Result<String, String> {
    let out = port::group_filter_draft(source, advanced)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupFilterDraft")]
pub fn group_filter_draft(source: String, advanced: Option<bool>) -> Result<String, JsError> {
    group_filter_draft_inner(source, advanced).map_err(|e| JsError::new(&e))
}

/// `groupFilterText`.
fn group_filter_text_inner(filter: String) -> Result<String, String> {
    let filter = crate::js::from_json(&filter)?;
    let out = port::group_filter_text(filter)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupFilterText")]
pub fn group_filter_text(filter: String) -> Result<String, JsError> {
    group_filter_text_inner(filter).map_err(|e| JsError::new(&e))
}

/// `groupFilterTexts`.
fn group_filter_texts_inner(filters: String) -> Result<String, String> {
    let filters = crate::js::from_json(&filters)?;
    let out = port::group_filter_texts(filters)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupFilterTexts")]
pub fn group_filter_texts(filters: String) -> Result<String, JsError> {
    group_filter_texts_inner(filters).map_err(|e| JsError::new(&e))
}

/// `reconcileGroupFilters`.
fn reconcile_group_filters_inner(previous: String, texts: Vec<String>) -> Result<String, String> {
    let previous = crate::js::from_json(&previous)?;
    let out = port::reconcile_group_filters(previous, texts)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "reconcileGroupFilters")]
pub fn reconcile_group_filters(previous: String, texts: Vec<String>) -> Result<String, JsError> {
    reconcile_group_filters_inner(previous, texts).map_err(|e| JsError::new(&e))
}

/// `newGroupFilter`.
fn new_group_filter_inner() -> Result<String, String> {
    let out = port::new_group_filter()?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "newGroupFilter")]
pub fn new_group_filter() -> Result<String, JsError> {
    new_group_filter_inner().map_err(|e| JsError::new(&e))
}

/// `groupAdmits`.
fn group_admits_inner(filters: Vec<String>, node: String) -> Result<String, String> {
    let node = crate::js::from_json(&node)?;
    let out = port::group_admits(filters, node)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupAdmits")]
pub fn group_admits(filters: Vec<String>, node: String) -> Result<String, JsError> {
    group_admits_inner(filters, node).map_err(|e| JsError::new(&e))
}

/// `readGroupEntries`.
fn read_group_entries_inner(text: String) -> Result<String, String> {
    let out = port::read_group_entries(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "readGroupEntries")]
pub fn read_group_entries(text: String) -> Result<String, JsError> {
    read_group_entries_inner(text).map_err(|e| JsError::new(&e))
}

/// `quoteName`.
fn quote_name_inner(value: String) -> Result<String, String> {
    let out = port::quote_name(value)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "quoteName")]
pub fn quote_name(value: String) -> Result<String, JsError> {
    quote_name_inner(value).map_err(|e| JsError::new(&e))
}

/// `isWritableName`.
fn is_writable_name_inner(value: String) -> Result<String, String> {
    let out = port::is_writable_name(value)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "isWritableName")]
pub fn is_writable_name(value: String) -> Result<String, JsError> {
    is_writable_name_inner(value).map_err(|e| JsError::new(&e))
}

/// `groupNameProblem`.
fn group_name_problem_inner(name: String, taken: String) -> Result<String, String> {
    let taken = crate::js::from_json(&taken)?;
    let out = port::group_name_problem(name, taken)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupNameProblem")]
pub fn group_name_problem(name: String, taken: String) -> Result<String, JsError> {
    group_name_problem_inner(name, taken).map_err(|e| JsError::new(&e))
}

/// `writeGroupEntry`.
fn write_group_entry_inner(text: String, name: String, next: String) -> Result<String, String> {
    let next = crate::js::from_json(&next)?;
    let out = port::write_group_entry(text, name, next)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeGroupEntry")]
pub fn write_group_entry(text: String, name: String, next: String) -> Result<String, JsError> {
    write_group_entry_inner(text, name, next).map_err(|e| JsError::new(&e))
}

/// `nameText`.
fn name_text_inner(value: String, written: String) -> Result<String, String> {
    let value = crate::js::from_json(&value)?;
    let written = crate::js::from_json(&written)?;
    let out = port::name_text(value, written)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "nameText")]
pub fn name_text(value: String, written: String) -> Result<String, JsError> {
    name_text_inner(value, written).map_err(|e| JsError::new(&e))
}

/// `nestedIn`.
fn nested_in_inner(entry: String) -> Result<Vec<String>, String> {
    let entry = crate::js::from_json(&entry)?;
    let out = port::nested_in(entry)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "nestedIn")]
pub fn nested_in(entry: String) -> Result<Vec<String>, JsError> {
    nested_in_inner(entry).map_err(|e| JsError::new(&e))
}

/// `exactTokens`.
fn exact_tokens_inner(filter: String, call: String) -> Result<String, String> {
    let call = crate::js::from_json(&call)?;
    let out = port::exact_tokens(filter, call)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "exactTokens")]
pub fn exact_tokens(filter: String, call: String) -> Result<String, JsError> {
    exact_tokens_inner(filter, call).map_err(|e| JsError::new(&e))
}

/// `namedIn`.
fn named_in_inner(entry: String) -> Result<Vec<String>, String> {
    let entry = crate::js::from_json(&entry)?;
    let out = port::named_in(entry)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "namedIn")]
pub fn named_in(entry: String) -> Result<Vec<String>, JsError> {
    named_in_inner(entry).map_err(|e| JsError::new(&e))
}

/// `classifyFilters`.
fn classify_filters_inner(entry: String) -> Result<String, String> {
    let entry = crate::js::from_json(&entry)?;
    let out = port::classify_filters(entry)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "classifyFilters")]
pub fn classify_filters(entry: String) -> Result<String, JsError> {
    classify_filters_inner(entry).map_err(|e| JsError::new(&e))
}

/// `describeFilters`.
fn describe_filters_inner(filters: Vec<String>) -> Result<String, String> {
    let out = port::describe_filters(filters)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "describeFilters")]
pub fn describe_filters(filters: Vec<String>) -> Result<String, JsError> {
    describe_filters_inner(filters).map_err(|e| JsError::new(&e))
}

/// `citingGroups`.
fn citing_groups_inner(text: String, tag: String) -> Result<String, String> {
    let out = port::citing_groups(text, tag)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "citingGroups")]
pub fn citing_groups(text: String, tag: String) -> Result<String, JsError> {
    citing_groups_inner(text, tag).map_err(|e| JsError::new(&e))
}

/// `namedInExpression`.
fn named_in_expression_inner(text: String, tag: String) -> Result<String, String> {
    let out = port::named_in_expression(text, tag)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "namedInExpression")]
pub fn named_in_expression(text: String, tag: String) -> Result<String, JsError> {
    named_in_expression_inner(text, tag).map_err(|e| JsError::new(&e))
}

/// `groupsNamingTag`.
fn groups_naming_tag_inner(text: String, tag: String) -> Result<String, String> {
    let out = port::groups_naming_tag(text, tag)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "groupsNamingTag")]
pub fn groups_naming_tag(text: String, tag: String) -> Result<String, JsError> {
    groups_naming_tag_inner(text, tag).map_err(|e| JsError::new(&e))
}

/// `addSubtagsToGroup`.
fn add_subtags_to_group_inner(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<String, String> {
    let out = port::add_subtags_to_group(text, group, tags)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "addSubtagsToGroup")]
pub fn add_subtags_to_group(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<String, JsError> {
    add_subtags_to_group_inner(text, group, tags).map_err(|e| JsError::new(&e))
}

/// `addNodesToGroup`.
fn add_nodes_to_group_inner(
    text: String,
    group: String,
    names: Vec<String>,
) -> Result<String, String> {
    let out = port::add_nodes_to_group(text, group, names)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "addNodesToGroup")]
pub fn add_nodes_to_group(
    text: String,
    group: String,
    names: Vec<String>,
) -> Result<String, JsError> {
    add_nodes_to_group_inner(text, group, names).map_err(|e| JsError::new(&e))
}

/// `removeSubtagsFromGroup`.
fn remove_subtags_from_group_inner(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<String, String> {
    let out = port::remove_subtags_from_group(text, group, tags)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "removeSubtagsFromGroup")]
pub fn remove_subtags_from_group(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<String, JsError> {
    remove_subtags_from_group_inner(text, group, tags).map_err(|e| JsError::new(&e))
}

/// `splitTop`.
fn split_top_inner(text: String, separator: String) -> Result<Vec<String>, String> {
    let out = port::split_top(text, separator)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "splitTop")]
pub fn split_top(text: String, separator: String) -> Result<Vec<String>, JsError> {
    split_top_inner(text, separator).map_err(|e| JsError::new(&e))
}

/// `ruleCondition`.
fn rule_condition_inner(kind: String, value: String) -> Result<Option<String>, String> {
    let kind = crate::js::from_json(&kind)?;
    let out = port::rule_condition(kind, value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "ruleCondition")]
pub fn rule_condition(kind: String, value: String) -> Result<Option<String>, JsError> {
    rule_condition_inner(kind, value).map_err(|e| JsError::new(&e))
}

/// `includePatterns`.
fn include_patterns_inner(text: String) -> Result<Vec<String>, String> {
    let out = port::include_patterns(text)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "includePatterns")]
pub fn include_patterns(text: String) -> Result<Vec<String>, JsError> {
    include_patterns_inner(text).map_err(|e| JsError::new(&e))
}

/// `isGlob`.
fn is_glob_inner(pattern: String) -> Result<String, String> {
    let out = port::is_glob(pattern)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "isGlob")]
pub fn is_glob(pattern: String) -> Result<String, JsError> {
    is_glob_inner(pattern).map_err(|e| JsError::new(&e))
}

/// `globMatch`.
fn glob_match_inner(pattern: String, path: String) -> Result<bool, String> {
    let out = port::glob_match(pattern, path)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "globMatch")]
pub fn glob_match(pattern: String, path: String) -> Result<bool, JsError> {
    glob_match_inner(pattern, path).map_err(|e| JsError::new(&e))
}

/// `includedBy`.
fn included_by_inner(patterns: Vec<String>, path: String) -> Result<String, String> {
    let out = port::included_by(patterns, path)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "includedBy")]
pub fn included_by(patterns: Vec<String>, path: String) -> Result<String, JsError> {
    included_by_inner(patterns, path).map_err(|e| JsError::new(&e))
}

/// `resolveIncludePath`.
fn resolve_include_path_inner(base: Option<String>, path: String) -> Result<String, String> {
    let out = port::resolve_include_path(base, path)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "resolveIncludePath")]
pub fn resolve_include_path(base: Option<String>, path: String) -> Result<String, JsError> {
    resolve_include_path_inner(base, path).map_err(|e| JsError::new(&e))
}

/// `includeDirectory`.
fn include_directory_inner(patterns: Vec<String>) -> Result<String, String> {
    let out = port::include_directory(patterns)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "includeDirectory")]
pub fn include_directory(patterns: Vec<String>) -> Result<String, JsError> {
    include_directory_inner(patterns).map_err(|e| JsError::new(&e))
}

/// `newSourcePathProblem`.
fn new_source_path_problem_inner(path: String) -> Result<String, String> {
    let out = port::new_source_path_problem(path)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "newSourcePathProblem")]
pub fn new_source_path_problem(path: String) -> Result<String, JsError> {
    new_source_path_problem_inner(path).map_err(|e| JsError::new(&e))
}

/// `namePatterns`.
fn name_patterns_inner(patterns: Vec<String>) -> Result<String, String> {
    let out = port::name_patterns(patterns)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "namePatterns")]
pub fn name_patterns(patterns: Vec<String>) -> Result<String, JsError> {
    name_patterns_inner(patterns).map_err(|e| JsError::new(&e))
}

/// `newSourceNameProblem`.
fn new_source_name_problem_inner(name: String, pattern: String) -> Result<String, String> {
    let pattern = crate::js::from_json(&pattern)?;
    let out = port::new_source_name_problem(name, pattern)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "newSourceNameProblem")]
pub fn new_source_name_problem(name: String, pattern: String) -> Result<String, JsError> {
    new_source_name_problem_inner(name, pattern).map_err(|e| JsError::new(&e))
}

/// `sourceAt`.
fn source_at_inner(sources: String, path: String) -> Result<String, String> {
    let sources = crate::js::from_json(&sources)?;
    let out = port::source_at(sources, path)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "sourceAt")]
pub fn source_at(sources: String, path: String) -> Result<String, JsError> {
    source_at_inner(sources, path).map_err(|e| JsError::new(&e))
}

/// `isProtocol`.
fn is_protocol_inner(value: String) -> Result<bool, String> {
    let out = port::is_protocol(value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "isProtocol")]
pub fn is_protocol(value: String) -> Result<bool, JsError> {
    is_protocol_inner(value).map_err(|e| JsError::new(&e))
}

/// `protocolName`.
fn protocol_name_inner(protocol: String) -> Result<String, String> {
    let protocol = crate::js::from_json(&protocol)?;
    let out = port::protocol_name(protocol)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "protocolName")]
pub fn protocol_name(protocol: String) -> Result<String, JsError> {
    protocol_name_inner(protocol).map_err(|e| JsError::new(&e))
}

/// `emptyNode`.
fn empty_node_inner(protocol: String) -> Result<String, String> {
    let protocol = crate::js::from_json(&protocol)?;
    let out = port::empty_node(protocol)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "emptyNode")]
pub fn empty_node(protocol: String) -> Result<String, JsError> {
    empty_node_inner(protocol).map_err(|e| JsError::new(&e))
}

/// `parseNodeLink`.
fn parse_node_link_inner(original: String) -> Result<String, String> {
    let out = port::parse_node_link(original)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "parseNodeLink")]
pub fn parse_node_link(original: String) -> Result<String, JsError> {
    parse_node_link_inner(original).map_err(|e| JsError::new(&e))
}

/// `nodeLink`.
fn node_link_inner(fields: String, document: String) -> Result<String, String> {
    let fields = crate::js::from_json(&fields)?;
    let document = crate::js::from_json(&document)?;
    let out = port::node_link(fields, document)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "nodeLink")]
pub fn node_link(fields: String, document: String) -> Result<String, JsError> {
    node_link_inner(fields, document).map_err(|e| JsError::new(&e))
}

/// `securityOptions`.
fn security_options_inner(protocol: String) -> Result<String, String> {
    let protocol = crate::js::from_json(&protocol)?;
    let out = port::security_options(protocol)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "securityOptions")]
pub fn security_options(protocol: String) -> Result<String, JsError> {
    security_options_inner(protocol).map_err(|e| JsError::new(&e))
}

/// `usesTls`.
fn uses_tls_inner(f: String) -> Result<String, String> {
    let f = crate::js::from_json(&f)?;
    let out = port::uses_tls(f)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "usesTls")]
pub fn uses_tls(f: String) -> Result<String, JsError> {
    uses_tls_inner(f).map_err(|e| JsError::new(&e))
}

/// `manualFields`.
fn manual_fields_inner(f: String) -> Result<String, String> {
    let f = crate::js::from_json(&f)?;
    let out = port::manual_fields(f)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "manualFields")]
pub fn manual_fields(f: String) -> Result<String, JsError> {
    manual_fields_inner(f).map_err(|e| JsError::new(&e))
}

/// `advancedFields`.
fn advanced_fields_inner(f: String) -> Result<String, String> {
    let f = crate::js::from_json(&f)?;
    let out = port::advanced_fields(f)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "advancedFields")]
pub fn advanced_fields(f: String) -> Result<String, JsError> {
    advanced_fields_inner(f).map_err(|e| JsError::new(&e))
}

/// `settle`.
fn settle_inner(f: String) -> Result<String, String> {
    let f = crate::js::from_json(&f)?;
    let out = port::settle(f)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "settle")]
pub fn settle(f: String) -> Result<String, JsError> {
    settle_inner(f).map_err(|e| JsError::new(&e))
}

/// `manualInvalid`.
fn manual_invalid_inner(f: String) -> Result<Vec<String>, String> {
    let f = crate::js::from_json(&f)?;
    let out = port::manual_invalid(f)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "manualInvalid")]
pub fn manual_invalid(f: String) -> Result<Vec<String>, JsError> {
    manual_invalid_inner(f).map_err(|e| JsError::new(&e))
}

/// `manualLink`.
fn manual_link_inner(f: String) -> Result<String, String> {
    let f = crate::js::from_json(&f)?;
    let out = port::manual_link(f)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "manualLink")]
pub fn manual_link(f: String) -> Result<String, JsError> {
    manual_link_inner(f).map_err(|e| JsError::new(&e))
}

/// `isNodeLink`.
fn is_node_link_inner(value: String) -> Result<String, String> {
    let out = port::is_node_link(value)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "isNodeLink")]
pub fn is_node_link(value: String) -> Result<String, JsError> {
    is_node_link_inner(value).map_err(|e| JsError::new(&e))
}

/// `readNodeEntries`.
fn read_node_entries_inner(text: String) -> Result<String, String> {
    let out = port::read_node_entries(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "readNodeEntries")]
pub fn read_node_entries(text: String) -> Result<String, JsError> {
    read_node_entries_inner(text).map_err(|e| JsError::new(&e))
}

/// `readChainEntries`.
fn read_chain_entries_inner(text: String) -> Result<String, String> {
    let out = port::read_chain_entries(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "readChainEntries")]
pub fn read_chain_entries(text: String) -> Result<String, JsError> {
    read_chain_entries_inner(text).map_err(|e| JsError::new(&e))
}

/// `nodeReferenced`.
fn node_referenced_inner(text: String, name: String) -> Result<String, String> {
    let out = port::node_referenced(text, name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "nodeReferenced")]
pub fn node_referenced(text: String, name: String) -> Result<String, JsError> {
    node_referenced_inner(text, name).map_err(|e| JsError::new(&e))
}

/// `nodeReferrers`.
fn node_referrers_inner(text: String, name: String) -> Result<String, String> {
    let out = port::node_referrers(text, name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "nodeReferrers")]
pub fn node_referrers(text: String, name: String) -> Result<String, JsError> {
    node_referrers_inner(text, name).map_err(|e| JsError::new(&e))
}

/// `removeChainEntry`.
fn remove_chain_entry_inner(text: String, original: String) -> Result<String, String> {
    let original = crate::js::from_json(&original)?;
    let out = port::remove_chain_entry(text, original)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "removeChainEntry")]
pub fn remove_chain_entry(text: String, original: String) -> Result<String, JsError> {
    remove_chain_entry_inner(text, original).map_err(|e| JsError::new(&e))
}

/// `chainHopCandidate`.
fn chain_hop_candidate_inner(node: String) -> Result<String, String> {
    let node = crate::js::from_json(&node)?;
    let out = port::chain_hop_candidate(node)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "chainHopCandidate")]
pub fn chain_hop_candidate(node: String) -> Result<String, JsError> {
    chain_hop_candidate_inner(node).map_err(|e| JsError::new(&e))
}

/// `tunnelDialable`.
fn tunnel_dialable_inner(node: String) -> Result<String, String> {
    let node = crate::js::from_json(&node)?;
    let out = port::tunnel_dialable(node)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "tunnelDialable")]
pub fn tunnel_dialable(node: String) -> Result<String, JsError> {
    tunnel_dialable_inner(node).map_err(|e| JsError::new(&e))
}

/// `chainHopProblem`.
fn chain_hop_problem_inner(
    nodes_list: String,
    name: String,
    index: f64,
) -> Result<Option<String>, String> {
    let nodes_list = crate::js::from_json(&nodes_list)?;
    let out = port::chain_hop_problem(nodes_list, name, index)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "chainHopProblem")]
pub fn chain_hop_problem(
    nodes_list: String,
    name: String,
    index: f64,
) -> Result<Option<String>, JsError> {
    chain_hop_problem_inner(nodes_list, name, index).map_err(|e| JsError::new(&e))
}

/// `chainUdpProblem`.
fn chain_udp_problem_inner(hops: String) -> Result<String, String> {
    let hops = crate::js::from_json(&hops)?;
    let out = port::chain_udp_problem(hops)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "chainUdpProblem")]
pub fn chain_udp_problem(hops: String) -> Result<String, JsError> {
    chain_udp_problem_inner(hops).map_err(|e| JsError::new(&e))
}

/// `writeChainEntry`.
fn write_chain_entry_inner(text: String, original: String, next: String) -> Result<String, String> {
    let original = crate::js::from_json(&original)?;
    let next = crate::js::from_json(&next)?;
    let out = port::write_chain_entry(text, original, next)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeChainEntry")]
pub fn write_chain_entry(text: String, original: String, next: String) -> Result<String, JsError> {
    write_chain_entry_inner(text, original, next).map_err(|e| JsError::new(&e))
}

/// `appendChainEntry`.
fn append_chain_entry_inner(text: String, next: String) -> Result<String, String> {
    let next = crate::js::from_json(&next)?;
    let out = port::append_chain_entry(text, next)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "appendChainEntry")]
pub fn append_chain_entry(text: String, next: String) -> Result<String, JsError> {
    append_chain_entry_inner(text, next).map_err(|e| JsError::new(&e))
}

/// `writeNodeEntry`.
fn write_node_entry_inner(text: String, original: String, next: String) -> Result<String, String> {
    let original = crate::js::from_json(&original)?;
    let next = crate::js::from_json(&next)?;
    let out = port::write_node_entry(text, original, next)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeNodeEntry")]
pub fn write_node_entry(text: String, original: String, next: String) -> Result<String, JsError> {
    write_node_entry_inner(text, original, next).map_err(|e| JsError::new(&e))
}

/// `readRuleFallback`.
fn read_rule_fallback_inner(text: String) -> Result<String, String> {
    let out = port::read_rule_fallback(text)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "readRuleFallback")]
pub fn read_rule_fallback(text: String) -> Result<String, JsError> {
    read_rule_fallback_inner(text).map_err(|e| JsError::new(&e))
}

/// `writeRuleFallback`.
fn write_rule_fallback_inner(text: String, target: String) -> Result<String, String> {
    let out = port::write_rule_fallback(text, target)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeRuleFallback")]
pub fn write_rule_fallback(text: String, target: String) -> Result<String, JsError> {
    write_rule_fallback_inner(text, target).map_err(|e| JsError::new(&e))
}

/// `readMode`.
fn read_mode_inner(text: String) -> Result<String, String> {
    let out = port::read_mode(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "readMode")]
pub fn read_mode(text: String) -> Result<String, JsError> {
    read_mode_inner(text).map_err(|e| JsError::new(&e))
}

/// `writeMode`.
fn write_mode_inner(text: String, next: String) -> Result<String, String> {
    let next = crate::js::from_json(&next)?;
    let out = port::write_mode(text, next)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeMode")]
pub fn write_mode(text: String, next: String) -> Result<String, JsError> {
    write_mode_inner(text, next).map_err(|e| JsError::new(&e))
}

/// `sameMode`.
fn same_mode_inner(a: String, b: String) -> Result<String, String> {
    let a = crate::js::from_json(&a)?;
    let b = crate::js::from_json(&b)?;
    let out = port::same_mode(a, b)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "sameMode")]
pub fn same_mode(a: String, b: String) -> Result<String, JsError> {
    same_mode_inner(a, b).map_err(|e| JsError::new(&e))
}

/// `modeRuleLines`.
fn mode_rule_lines_inner(text: String) -> Result<String, String> {
    let out = port::mode_rule_lines(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "modeRuleLines")]
pub fn mode_rule_lines(text: String) -> Result<String, JsError> {
    mode_rule_lines_inner(text).map_err(|e| JsError::new(&e))
}

/// `nodeProbeSupport`.
fn node_probe_support_inner(protocol: Option<String>) -> Result<String, String> {
    let out = port::node_probe_support(protocol)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "nodeProbeSupport")]
pub fn node_probe_support(protocol: Option<String>) -> Result<String, JsError> {
    node_probe_support_inner(protocol).map_err(|e| JsError::new(&e))
}

/// `supportsNodeProbe`.
fn supports_node_probe_inner(
    protocol: Option<String>,
    kind: String,
    transports: String,
) -> Result<bool, String> {
    let kind = crate::js::from_json(&kind)?;
    let transports = crate::js::from_json(&transports)?;
    let out = port::supports_node_probe(protocol, kind, transports)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "supportsNodeProbe")]
pub fn supports_node_probe(
    protocol: Option<String>,
    kind: String,
    transports: String,
) -> Result<bool, JsError> {
    supports_node_probe_inner(protocol, kind, transports).map_err(|e| JsError::new(&e))
}

/// `regionFlag`.
fn region_flag_inner(region: String) -> Result<String, String> {
    let out = port::region_flag(region)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "regionFlag")]
pub fn region_flag(region: String) -> Result<String, JsError> {
    region_flag_inner(region).map_err(|e| JsError::new(&e))
}

/// `matchRegion`.
fn match_region_inner(name: String) -> Result<String, String> {
    let out = port::match_region(name)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "matchRegion")]
pub fn match_region(name: String) -> Result<String, JsError> {
    match_region_inner(name).map_err(|e| JsError::new(&e))
}

/// `sourceFor`.
fn source_for_inner(list: String, source: String) -> Result<String, String> {
    let list = crate::js::from_json(&list)?;
    let source = crate::js::from_json(&source)?;
    let out = port::source_for(list, source)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "sourceFor")]
pub fn source_for(list: String, source: String) -> Result<String, JsError> {
    source_for_inner(list, source).map_err(|e| JsError::new(&e))
}

/// `ruleAnchor`.
fn rule_anchor_inner(source: String, rule: String, scan: String) -> Result<String, String> {
    let source = crate::js::from_json(&source)?;
    let rule = crate::js::from_json(&rule)?;
    let scan = crate::js::from_json(&scan)?;
    let out = port::rule_anchor(source, rule, scan)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "ruleAnchor")]
pub fn rule_anchor(source: String, rule: String, scan: String) -> Result<String, JsError> {
    rule_anchor_inner(source, rule, scan).map_err(|e| JsError::new(&e))
}

/// `dnsRuleTarget`.
fn dns_rule_target_inner(rule: String) -> Result<String, String> {
    let rule = crate::js::from_json(&rule)?;
    let out = port::dns_rule_target(rule)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "dnsRuleTarget")]
pub fn dns_rule_target(rule: String) -> Result<String, JsError> {
    dns_rule_target_inner(rule).map_err(|e| JsError::new(&e))
}

/// `dnsRuleAnchor`.
fn dns_rule_anchor_inner(
    source: String,
    rule: String,
    list: String,
    scan: String,
) -> Result<String, String> {
    let source = crate::js::from_json(&source)?;
    let rule = crate::js::from_json(&rule)?;
    let scan = crate::js::from_json(&scan)?;
    let out = port::dns_rule_anchor(source, rule, list, scan)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "dnsRuleAnchor")]
pub fn dns_rule_anchor(
    source: String,
    rule: String,
    list: String,
    scan: String,
) -> Result<String, JsError> {
    dns_rule_anchor_inner(source, rule, list, scan).map_err(|e| JsError::new(&e))
}

/// `dnsListEnd`.
fn dns_list_end_inner(sources: String, list: String) -> Result<String, String> {
    let sources = crate::js::from_json(&sources)?;
    let out = port::dns_list_end(sources, list)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "dnsListEnd")]
pub fn dns_list_end(sources: String, list: String) -> Result<String, JsError> {
    dns_list_end_inner(sources, list).map_err(|e| JsError::new(&e))
}

/// `dnsUpstreams`.
fn dns_upstreams_inner(text: String, scan: String) -> Result<String, String> {
    let scan = crate::js::from_json(&scan)?;
    let out = port::dns_upstreams(text, scan)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "dnsUpstreams")]
pub fn dns_upstreams(text: String, scan: String) -> Result<String, JsError> {
    dns_upstreams_inner(text, scan).map_err(|e| JsError::new(&e))
}

/// `dnsUpstreamNames`.
fn dns_upstream_names_inner(text: String, scan: String) -> Result<Vec<String>, String> {
    let scan = crate::js::from_json(&scan)?;
    let out = port::dns_upstream_names(text, scan)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "dnsUpstreamNames")]
pub fn dns_upstream_names(text: String, scan: String) -> Result<Vec<String>, JsError> {
    dns_upstream_names_inner(text, scan).map_err(|e| JsError::new(&e))
}

/// `ruleLine`.
fn rule_line_inner(
    condition: String,
    outbound: String,
    must: Option<bool>,
) -> Result<String, String> {
    let out = port::rule_line(condition, outbound, must)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "ruleLine")]
pub fn rule_line(
    condition: String,
    outbound: String,
    must: Option<bool>,
) -> Result<String, JsError> {
    rule_line_inner(condition, outbound, must).map_err(|e| JsError::new(&e))
}

/// `addRule`.
fn add_rule_inner(
    text: String,
    anchor: String,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    let anchor = crate::js::from_json(&anchor)?;
    let out = port::add_rule(text, anchor, condition, outbound, must)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "addRule")]
pub fn add_rule(
    text: String,
    anchor: String,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, JsError> {
    add_rule_inner(text, anchor, condition, outbound, must).map_err(|e| JsError::new(&e))
}

/// `replaceRuleTarget`.
fn replace_rule_target_inner(
    text: String,
    anchor: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    let anchor = crate::js::from_json(&anchor)?;
    let out = port::replace_rule_target(text, anchor, outbound, must)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "replaceRuleTarget")]
pub fn replace_rule_target(
    text: String,
    anchor: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, JsError> {
    replace_rule_target_inner(text, anchor, outbound, must).map_err(|e| JsError::new(&e))
}

/// `removeRule`.
fn remove_rule_inner(text: String, anchor: String) -> Result<Option<String>, String> {
    let anchor = crate::js::from_json(&anchor)?;
    let out = port::remove_rule(text, anchor)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "removeRule")]
pub fn remove_rule(text: String, anchor: String) -> Result<Option<String>, JsError> {
    remove_rule_inner(text, anchor).map_err(|e| JsError::new(&e))
}

/// `replaceRule`.
fn replace_rule_inner(
    text: String,
    anchor: String,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    let anchor = crate::js::from_json(&anchor)?;
    let out = port::replace_rule(text, anchor, condition, outbound, must)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "replaceRule")]
pub fn replace_rule(
    text: String,
    anchor: String,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, JsError> {
    replace_rule_inner(text, anchor, condition, outbound, must).map_err(|e| JsError::new(&e))
}

/// `addFallback`.
fn add_fallback_inner(
    text: String,
    anchor: String,
    target: String,
) -> Result<Option<String>, String> {
    let anchor = crate::js::from_json(&anchor)?;
    let out = port::add_fallback(text, anchor, target)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "addFallback")]
pub fn add_fallback(
    text: String,
    anchor: String,
    target: String,
) -> Result<Option<String>, JsError> {
    add_fallback_inner(text, anchor, target).map_err(|e| JsError::new(&e))
}

/// `numberSetting`.
fn number_setting_inner(field: String) -> Result<bool, String> {
    let field = crate::js::from_json(&field)?;
    let out = port::number_setting(field)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "numberSetting")]
pub fn number_setting(field: String) -> Result<bool, JsError> {
    number_setting_inner(field).map_err(|e| JsError::new(&e))
}

/// `settingValue`.
fn setting_value_inner(field: String, value: String) -> Result<String, String> {
    let field = crate::js::from_json(&field)?;
    let out = port::setting_value(field, value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "settingValue")]
pub fn setting_value(field: String, value: String) -> Result<String, JsError> {
    setting_value_inner(field, value).map_err(|e| JsError::new(&e))
}

/// `serializeSetting`.
fn serialize_setting_inner(field: String, value: String) -> Result<Option<String>, String> {
    let field = crate::js::from_json(&field)?;
    let out = port::serialize_setting(field, value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "serializeSetting")]
pub fn serialize_setting(field: String, value: String) -> Result<Option<String>, JsError> {
    serialize_setting_inner(field, value).map_err(|e| JsError::new(&e))
}

/// `writeSettings`.
fn write_settings_inner(
    text: String,
    section: String,
    index: f64,
    patch: String,
) -> Result<String, String> {
    let section = crate::js::from_json(&section)?;
    let patch = crate::js::from_json(&patch)?;
    let out = port::write_settings(text, section, index, patch)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeSettings")]
pub fn write_settings(
    text: String,
    section: String,
    index: f64,
    patch: String,
) -> Result<String, JsError> {
    write_settings_inner(text, section, index, patch).map_err(|e| JsError::new(&e))
}

/// `holdsDns`.
fn holds_dns_inner(texts: Vec<String>) -> Result<bool, String> {
    let out = port::holds_dns(texts)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "holdsDns")]
pub fn holds_dns(texts: Vec<String>) -> Result<bool, JsError> {
    holds_dns_inner(texts).map_err(|e| JsError::new(&e))
}

/// `holdsRouting`.
fn holds_routing_inner(texts: Vec<String>) -> Result<bool, String> {
    let out = port::holds_routing(texts)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "holdsRouting")]
pub fn holds_routing(texts: Vec<String>) -> Result<bool, JsError> {
    holds_routing_inner(texts).map_err(|e| JsError::new(&e))
}

/// `templateGroups`.
fn template_groups_inner(rules: String, defined: Vec<String>) -> Result<Vec<String>, String> {
    let out = port::template_groups(rules, defined)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "templateGroups")]
pub fn template_groups(rules: String, defined: Vec<String>) -> Result<Vec<String>, JsError> {
    template_groups_inner(rules, defined).map_err(|e| JsError::new(&e))
}

/// `allGroupNames`.
fn all_group_names_inner(sources: String, draft: String) -> Result<String, String> {
    let sources = crate::js::from_json(&sources)?;
    let draft = crate::js::from_json(&draft)?;
    let out = port::all_group_names(sources, draft)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "allGroupNames")]
pub fn all_group_names(sources: String, draft: String) -> Result<String, JsError> {
    all_group_names_inner(sources, draft).map_err(|e| JsError::new(&e))
}

/// `allUpstreamNames`.
fn all_upstream_names_inner(sources: String, draft: String) -> Result<String, String> {
    let sources = crate::js::from_json(&sources)?;
    let draft = crate::js::from_json(&draft)?;
    let out = port::all_upstream_names(sources, draft)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "allUpstreamNames")]
pub fn all_upstream_names(sources: String, draft: String) -> Result<String, JsError> {
    all_upstream_names_inner(sources, draft).map_err(|e| JsError::new(&e))
}

/// `redacted`.
fn redacted_inner(source: String) -> Result<String, String> {
    let source = crate::js::from_json(&source)?;
    let out = port::redacted(source)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "redacted")]
pub fn redacted(source: String) -> Result<String, JsError> {
    redacted_inner(source).map_err(|e| JsError::new(&e))
}

/// `fileName`.
fn file_name_inner(source: String) -> Result<String, String> {
    let source = crate::js::from_json(&source)?;
    let out = port::file_name(source)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "fileName")]
pub fn file_name(source: String) -> Result<String, JsError> {
    file_name_inner(source).map_err(|e| JsError::new(&e))
}

/// `validationSources`.
fn validation_sources_inner(sources: String, replacement: String) -> Result<String, String> {
    let sources = crate::js::from_json(&sources)?;
    let replacement = crate::js::from_json(&replacement)?;
    let out = port::validation_sources(sources, replacement)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "validationSources")]
pub fn validation_sources(sources: String, replacement: String) -> Result<String, JsError> {
    validation_sources_inner(sources, replacement).map_err(|e| JsError::new(&e))
}

/// `restartSettings`.
fn restart_settings_inner(diagnostics: String) -> Result<String, String> {
    let diagnostics = crate::js::from_json(&diagnostics)?;
    let out = port::restart_settings(diagnostics)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "restartSettings")]
pub fn restart_settings(diagnostics: String) -> Result<String, JsError> {
    restart_settings_inner(diagnostics).map_err(|e| JsError::new(&e))
}

/// `restartRequired`.
fn restart_required_inner(diagnostics: String) -> Result<String, String> {
    let diagnostics = crate::js::from_json(&diagnostics)?;
    let out = port::restart_required(diagnostics)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "restartRequired")]
pub fn restart_required(diagnostics: String) -> Result<String, JsError> {
    restart_required_inner(diagnostics).map_err(|e| JsError::new(&e))
}

/// `unwrittenRestartSettings`.
fn unwritten_restart_settings_inner(
    settings_list: String,
    sources: String,
) -> Result<String, String> {
    let settings_list = crate::js::from_json(&settings_list)?;
    let sources = crate::js::from_json(&sources)?;
    let out = port::unwritten_restart_settings(settings_list, sources)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "unwrittenRestartSettings")]
pub fn unwritten_restart_settings(
    settings_list: String,
    sources: String,
) -> Result<String, JsError> {
    unwritten_restart_settings_inner(settings_list, sources).map_err(|e| JsError::new(&e))
}

/// `completeSubscriptionUrl`.
fn complete_subscription_url_inner(value: String) -> Result<String, String> {
    let out = port::complete_subscription_url(value)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "completeSubscriptionUrl")]
pub fn complete_subscription_url(value: String) -> Result<String, JsError> {
    complete_subscription_url_inner(value).map_err(|e| JsError::new(&e))
}

/// `isSubscriptionUrl`.
fn is_subscription_url_inner(value: String) -> Result<String, String> {
    let out = port::is_subscription_url(value)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "isSubscriptionUrl")]
pub fn is_subscription_url(value: String) -> Result<String, JsError> {
    is_subscription_url_inner(value).map_err(|e| JsError::new(&e))
}

/// `parseInterval`.
fn parse_interval_inner(value: String) -> Result<String, String> {
    let out = port::parse_interval(value)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "parseInterval")]
pub fn parse_interval(value: String) -> Result<String, JsError> {
    parse_interval_inner(value).map_err(|e| JsError::new(&e))
}

/// `urlHost`.
fn url_host_inner(url: Option<String>) -> Result<String, String> {
    let out = port::url_host(url)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "urlHost")]
pub fn url_host(url: Option<String>) -> Result<String, JsError> {
    url_host_inner(url).map_err(|e| JsError::new(&e))
}

/// `agentProblem`.
fn agent_problem_inner(agent: String, written: bool) -> Result<String, String> {
    let out = port::agent_problem(agent, written)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "agentProblem")]
pub fn agent_problem(agent: String, written: bool) -> Result<String, JsError> {
    agent_problem_inner(agent, written).map_err(|e| JsError::new(&e))
}

/// `readSubscriptionEntries`.
fn read_subscription_entries_inner(text: String) -> Result<String, String> {
    let out = port::read_subscription_entries(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "readSubscriptionEntries")]
pub fn read_subscription_entries(text: String) -> Result<String, JsError> {
    read_subscription_entries_inner(text).map_err(|e| JsError::new(&e))
}

/// `writeSubscriptionEntry`.
fn write_subscription_entry_inner(
    text: String,
    tag: String,
    change: String,
) -> Result<String, String> {
    let change = crate::js::from_json(&change)?;
    let out = port::write_subscription_entry(text, tag, change)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeSubscriptionEntry")]
pub fn write_subscription_entry(
    text: String,
    tag: String,
    change: String,
) -> Result<String, JsError> {
    write_subscription_entry_inner(text, tag, change).map_err(|e| JsError::new(&e))
}

/// `templateRules`.
fn template_rules_inner(template: String, options: String) -> Result<Vec<String>, String> {
    let options = crate::js::from_json(&options)?;
    let out = port::template_rules(template, options)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "templateRules")]
pub fn template_rules(template: String, options: String) -> Result<Vec<String>, JsError> {
    template_rules_inner(template, options).map_err(|e| JsError::new(&e))
}

/// `detectTemplate`.
fn detect_template_inner(text: String) -> Result<String, String> {
    let out = port::detect_template(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "detectTemplate")]
pub fn detect_template(text: String) -> Result<String, JsError> {
    detect_template_inner(text).map_err(|e| JsError::new(&e))
}

/// `routingOptions`.
fn routing_options_inner(text: String) -> Result<String, String> {
    let out = port::routing_options(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "routingOptions")]
pub fn routing_options(text: String) -> Result<String, JsError> {
    routing_options_inner(text).map_err(|e| JsError::new(&e))
}

/// `isPresetRouting`.
fn is_preset_routing_inner(text: String) -> Result<bool, String> {
    let out = port::is_preset_routing(text)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "isPresetRouting")]
pub fn is_preset_routing(text: String) -> Result<bool, JsError> {
    is_preset_routing_inner(text).map_err(|e| JsError::new(&e))
}

/// `isBareName`.
fn is_bare_name_inner(value: String) -> Result<String, String> {
    let out = port::is_bare_name(value)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "isBareName")]
pub fn is_bare_name(value: String) -> Result<String, JsError> {
    is_bare_name_inner(value).map_err(|e| JsError::new(&e))
}

/// `quote`.
fn quote_inner(value: String) -> Result<String, String> {
    let out = port::quote(value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "quote")]
pub fn quote(value: String) -> Result<String, JsError> {
    quote_inner(value).map_err(|e| JsError::new(&e))
}

/// `isQuotable`.
fn is_quotable_inner(value: String) -> Result<bool, String> {
    let out = port::is_quotable(value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "isQuotable")]
pub fn is_quotable(value: String) -> Result<bool, JsError> {
    is_quotable_inner(value).map_err(|e| JsError::new(&e))
}

/// `scanConfig`.
fn scan_config_inner(text: String) -> Result<String, String> {
    let out = port::scan_config(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "scanConfig")]
pub fn scan_config(text: String) -> Result<String, JsError> {
    scan_config_inner(text).map_err(|e| JsError::new(&e))
}

/// `isFragment`.
fn is_fragment_inner(text: String) -> Result<bool, String> {
    let out = port::is_fragment(text)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "isFragment")]
pub fn is_fragment(text: String) -> Result<bool, JsError> {
    is_fragment_inner(text).map_err(|e| JsError::new(&e))
}

/// `unquote`.
fn unquote_inner(text: String) -> Result<String, String> {
    let out = port::unquote(text)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "unquote")]
pub fn unquote(text: String) -> Result<String, JsError> {
    unquote_inner(text).map_err(|e| JsError::new(&e))
}

/// `uncomment`.
fn uncomment_inner(text: String) -> Result<String, String> {
    let out = port::uncomment(text)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "uncomment")]
pub fn uncomment(text: String) -> Result<String, JsError> {
    uncomment_inner(text).map_err(|e| JsError::new(&e))
}

/// `blockFields`.
fn block_fields_inner(text: String, block: String, tokens: String) -> Result<String, String> {
    let block = crate::js::from_json(&block)?;
    let tokens = crate::js::from_json(&tokens)?;
    let out = port::block_fields(text, block, tokens)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "blockFields")]
pub fn block_fields(text: String, block: String, tokens: String) -> Result<String, JsError> {
    block_fields_inner(text, block, tokens).map_err(|e| JsError::new(&e))
}

/// `blockBody`.
fn block_body_inner(text: String, block: String) -> Result<Vec<String>, String> {
    let block = crate::js::from_json(&block)?;
    let out = port::block_body(text, block)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "blockBody")]
pub fn block_body(text: String, block: String) -> Result<Vec<String>, JsError> {
    block_body_inner(text, block).map_err(|e| JsError::new(&e))
}

/// `blockEntries`.
fn block_entries_inner(text: String, block: String) -> Result<String, String> {
    let block = crate::js::from_json(&block)?;
    let out = port::block_entries(text, block)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "blockEntries")]
pub fn block_entries(text: String, block: String) -> Result<String, JsError> {
    block_entries_inner(text, block).map_err(|e| JsError::new(&e))
}

/// `policyKind`.
fn policy_kind_inner(native: String) -> Result<String, String> {
    let out = port::policy_kind(native)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "policyKind")]
pub fn policy_kind(native: String) -> Result<String, JsError> {
    policy_kind_inner(native).map_err(|e| JsError::new(&e))
}

/// `isBuiltinOutbound`.
fn is_builtin_outbound_inner(name: Option<String>) -> Result<bool, String> {
    let out = port::is_builtin_outbound(name)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "isBuiltinOutbound")]
pub fn is_builtin_outbound(name: Option<String>) -> Result<bool, JsError> {
    is_builtin_outbound_inner(name).map_err(|e| JsError::new(&e))
}

/// `readXhttp`.
fn read_xhttp_inner(link: String) -> Result<String, String> {
    let out = port::read_xhttp(link)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "readXhttp")]
pub fn read_xhttp(link: String) -> Result<String, JsError> {
    read_xhttp_inner(link).map_err(|e| JsError::new(&e))
}

/// `duplicateXhttpHeaders`.
fn duplicate_xhttp_headers_inner(headers: String) -> Result<String, String> {
    let headers = crate::js::from_json(&headers)?;
    let out = port::duplicate_xhttp_headers(headers)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "duplicateXhttpHeaders")]
pub fn duplicate_xhttp_headers(headers: String) -> Result<String, JsError> {
    duplicate_xhttp_headers_inner(headers).map_err(|e| JsError::new(&e))
}

/// `writeXhttp`.
fn write_xhttp_inner(link: String, change: String) -> Result<String, String> {
    let change = crate::js::from_json(&change)?;
    let out = port::write_xhttp(link, change)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "writeXhttp")]
pub fn write_xhttp(link: String, change: String) -> Result<String, JsError> {
    write_xhttp_inner(link, change).map_err(|e| JsError::new(&e))
}

// ---- the binary wire ------------------------------------------------------------------------

thread_local! {
    static WIRE_IN: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    static WIRE_OUT: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// This module's memory, where `index.cjs` reads and writes the wire buffers.
#[wasm_bindgen(js_name = "rustAnythingWireMemory")]
pub fn rust_anything_wire_memory() -> JsValue {
    wasm_bindgen::memory()
}

/// The address of an argument buffer of `len` bytes.
#[wasm_bindgen(js_name = "rustAnythingWireIn")]
pub fn rust_anything_wire_in(len: u32) -> u32 {
    WIRE_IN.with(|b| {
        let mut b = b.borrow_mut();
        b.resize(len as usize, 0);
        b.as_mut_ptr() as usize as u32
    })
}

/// The address of the last result.
#[wasm_bindgen(js_name = "rustAnythingWireOut")]
pub fn rust_anything_wire_out() -> u32 {
    WIRE_OUT.with(|b| b.borrow().as_ptr() as usize as u32)
}

/// Reads `len` argument bytes, runs `f`, and keeps its result for `rustAnythingWireOut`.
fn wire_io(
    len: u32,
    f: impl FnOnce(&mut wire::Reader, &mut wire::Writer) -> Result<(), String>,
) -> Result<u32, JsError> {
    let input = WIRE_IN.with(|b| std::mem::take(&mut *b.borrow_mut()));
    let mut w = wire::Writer::default();
    let result = f(
        &mut wire::Reader::new(&input[..(len as usize).min(input.len())]),
        &mut w,
    );
    WIRE_IN.with(|b| *b.borrow_mut() = input);
    match result {
        // A result of another shape than the TS type: JS runs the call through JSON.
        Err(e) if e.starts_with(wire::MISMATCH) => return Ok(u32::MAX),
        other => other.map_err(|e| JsError::new(&e))?,
    }
    let n = w.0.len();
    WIRE_OUT.with(|b| *b.borrow_mut() = w.0);
    Ok(u32::try_from(n).unwrap_or(u32::MAX))
}

static S_SERIALIZE_CONDITIONS_0: wire::Schema = wire::Schema::Dyn;

/// `serializeConditions` over the binary wire.
#[wasm_bindgen(js_name = "serializeConditionsWire")]
pub fn serialize_conditions_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let rows: serde_json::Value = wire::read_value(r, &S_SERIALIZE_CONDITIONS_0)?;
        let out = &port::serialize_conditions(rows)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_PARSE_CONDITIONS_OUT: wire::Schema =
    wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
        ("kind", wire::Schema::Dyn),
        ("value", wire::Schema::Str16),
        ("negate", wire::Schema::Bool),
    ])));

/// `parseConditions` over the binary wire.
#[wasm_bindgen(js_name = "parseConditionsWire")]
pub fn parse_conditions_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let kinds: Vec<String> = r.list(|r| {
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))
        })?;
        let out = &port::parse_conditions(text, kinds)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.tag(1);
                w.count(x0.len());
                for x1 in x0.iter() {
                    wire::write_value(w, x1, S_PARSE_CONDITIONS_OUT.inner().inner())?;
                }
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_FLAG_KEY_OUT: wire::Schema = wire::Schema::Dyn;

/// `flagKey` over the binary wire.
#[wasm_bindgen(js_name = "flagKeyWire")]
pub fn flag_key_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let name: String = r.string16()?;
        let out = &port::flag_key(name)?;
        wire::write_value(w, out, &S_FLAG_KEY_OUT)?;
        Ok(())
    })
}

static S_VALID_FLAG_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `validFlagName` over the binary wire.
#[wasm_bindgen(js_name = "validFlagNameWire")]
pub fn valid_flag_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let name: String = r.string16()?;
        let out = &port::valid_flag_name(name)?;
        wire::write_value(w, out, &S_VALID_FLAG_NAME_OUT)?;
        Ok(())
    })
}

static S_HAS_EMBEDDED_FLAG_OUT: wire::Schema = wire::Schema::Dyn;

/// `hasEmbeddedFlag` over the binary wire.
#[wasm_bindgen(js_name = "hasEmbeddedFlagWire")]
pub fn has_embedded_flag_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let name: String = r.string16()?;
        let out = &port::has_embedded_flag(name)?;
        wire::write_value(w, out, &S_HAS_EMBEDDED_FLAG_OUT)?;
        Ok(())
    })
}

static S_VALID_FLAG_CHOICE_0: wire::Schema = wire::Schema::Dyn;

/// `validFlagChoice` over the binary wire.
#[wasm_bindgen(js_name = "validFlagChoiceWire")]
pub fn valid_flag_choice_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: serde_json::Value = wire::read_value(r, &S_VALID_FLAG_CHOICE_0)?;
        let out = &port::valid_flag_choice(value)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_FLAG_CHOICES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("id", wire::Schema::Str16),
    ("label", wire::Schema::Str16),
    ("flag", wire::Schema::Str16),
    ("keywords", wire::Schema::Str16),
]));

/// `flagChoices` over the binary wire.
#[wasm_bindgen(js_name = "flagChoicesWire")]
pub fn flag_choices_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let locale: String = r.string16()?;
        let out = &port::flag_choices(locale)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_FLAG_CHOICES_OUT.inner())?;
        }
        Ok(())
    })
}

static S_MATCH_PRESET_0: wire::Schema = wire::Schema::Dyn;

/// `matchPreset` over the binary wire.
#[wasm_bindgen(js_name = "matchPresetWire")]
pub fn match_preset_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let urls: serde_json::Value = wire::read_value(r, &S_MATCH_PRESET_0)?;
        let out = &port::match_preset(urls)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

/// `missingCategories` over the binary wire.
#[wasm_bindgen(js_name = "missingCategoriesWire")]
pub fn missing_categories_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let preset: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let required: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::missing_categories(preset, required)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_GEODATA_PRESET_OUT: wire::Schema = wire::Schema::Dyn;

/// `geodataPreset` over the binary wire.
#[wasm_bindgen(js_name = "geodataPresetWire")]
pub fn geodata_preset_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let id: String = r.string16()?;
        let out = &port::geodata_preset(id)?;
        wire::write_value(w, out, &S_GEODATA_PRESET_OUT)?;
        Ok(())
    })
}

static S_CONDITION_FAMILY_0: wire::Schema = wire::Schema::Dyn;
static S_CONDITION_FAMILY_OUT: wire::Schema = wire::Schema::Dyn;

/// `conditionFamily` over the binary wire.
#[wasm_bindgen(js_name = "conditionFamilyWire")]
pub fn condition_family_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let kind: serde_json::Value = wire::read_value(r, &S_CONDITION_FAMILY_0)?;
        let out = &port::condition_family(kind)?;
        wire::write_value(w, out, &S_CONDITION_FAMILY_OUT)?;
        Ok(())
    })
}

static S_NEW_GROUP_CONDITION_OUT: wire::Schema = wire::Schema::Obj(&[
    ("id", wire::Schema::Num),
    ("kind", wire::Schema::Dyn),
    ("value", wire::Schema::Str16),
    ("negate", wire::Schema::Bool),
    (
        "alternatives",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
            ("id", wire::Schema::Num),
            ("kind", wire::Schema::Dyn),
            ("value", wire::Schema::Str16),
        ]))),
    ),
]);

/// `newGroupCondition` over the binary wire.
#[wasm_bindgen(js_name = "newGroupConditionWire")]
pub fn new_group_condition_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |_r, w| {
        let out = &port::new_group_condition()?;
        wire::write_value(w, out, &S_NEW_GROUP_CONDITION_OUT)?;
        Ok(())
    })
}

static S_CONDITION_VALUES_OUT: wire::Schema = wire::Schema::Dyn;

/// `conditionValues` over the binary wire.
#[wasm_bindgen(js_name = "conditionValuesWire")]
pub fn condition_values_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let values: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::condition_values(values)?;
        wire::write_value(w, out, &S_CONDITION_VALUES_OUT)?;
        Ok(())
    })
}

/// `parseConditionValues` over the binary wire.
#[wasm_bindgen(js_name = "parseConditionValuesWire")]
pub fn parse_condition_values_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::parse_condition_values(text)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.tag(1);
                w.count(x0.len());
                for x1 in x0.iter() {
                    w.string16(x1);
                }
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_PARSE_GROUP_CONDITIONS_OUT: wire::Schema =
    wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
        ("negate", wire::Schema::Bool),
        (
            "alternatives",
            wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
                ("id", wire::Schema::Num),
                ("kind", wire::Schema::Dyn),
                ("value", wire::Schema::Str16),
            ]))),
        ),
    ])));

/// `parseGroupConditions` over the binary wire.
#[wasm_bindgen(js_name = "parseGroupConditionsWire")]
pub fn parse_group_conditions_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let source: String = r.string16()?;
        let out = &port::parse_group_conditions(source)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.tag(1);
                w.count(x0.len());
                for x1 in x0.iter() {
                    wire::write_value(w, x1, S_PARSE_GROUP_CONDITIONS_OUT.inner().inner())?;
                }
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_SERIALIZE_GROUP_CONDITIONS_0: wire::Schema = wire::Schema::Dyn;

/// `serializeGroupConditions` over the binary wire.
#[wasm_bindgen(js_name = "serializeGroupConditionsWire")]
pub fn serialize_group_conditions_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let rows: serde_json::Value = wire::read_value(r, &S_SERIALIZE_GROUP_CONDITIONS_0)?;
        let out = &port::serialize_group_conditions(rows)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_GROUP_FILTER_DRAFT_OUT: wire::Schema = wire::Schema::Obj(&[
    ("id", wire::Schema::Num),
    ("source", wire::Schema::Str16),
    ("initial", wire::Schema::Str16),
    (
        "rows",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
            ("negate", wire::Schema::Bool),
            (
                "alternatives",
                wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
                    ("id", wire::Schema::Num),
                    ("kind", wire::Schema::Dyn),
                    ("value", wire::Schema::Str16),
                ]))),
            ),
        ]))),
    ),
    ("advanced", wire::Schema::Bool),
]);

/// `groupFilterDraft` over the binary wire.
#[wasm_bindgen(js_name = "groupFilterDraftWire")]
pub fn group_filter_draft_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let source: String = r.string16()?;
        let advanced: Option<bool> = r.opt_js(|r| r.bool())?;
        let out = &port::group_filter_draft(source, advanced)?;
        wire::write_value(w, out, &S_GROUP_FILTER_DRAFT_OUT)?;
        Ok(())
    })
}

static S_GROUP_FILTER_TEXT_0: wire::Schema = wire::Schema::Dyn;
static S_GROUP_FILTER_TEXT_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `groupFilterText` over the binary wire.
#[wasm_bindgen(js_name = "groupFilterTextWire")]
pub fn group_filter_text_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filter: serde_json::Value = wire::read_value(r, &S_GROUP_FILTER_TEXT_0)?;
        let out = &port::group_filter_text(filter)?;
        wire::write_value(w, out, &S_GROUP_FILTER_TEXT_OUT)?;
        Ok(())
    })
}

static S_GROUP_FILTER_TEXTS_0: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("id", wire::Schema::Num),
    ("source", wire::Schema::Str16),
    ("initial", wire::Schema::Str16),
    (
        "rows",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
            ("negate", wire::Schema::Bool),
            (
                "alternatives",
                wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
                    ("id", wire::Schema::Dyn),
                    ("kind", wire::Schema::Dyn),
                    ("value", wire::Schema::Dyn),
                ]))),
            ),
        ]))),
    ),
    ("advanced", wire::Schema::Bool),
]));
static S_GROUP_FILTER_TEXTS_OUT: wire::Schema = wire::Schema::Dyn;

/// `groupFilterTexts` over the binary wire.
#[wasm_bindgen(js_name = "groupFilterTextsWire")]
pub fn group_filter_texts_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filters: Vec<serde_json::Value> =
            r.list(|r| wire::read_value(r, S_GROUP_FILTER_TEXTS_0.inner()))?;
        let out = &port::group_filter_texts(filters)?;
        wire::write_value(w, out, &S_GROUP_FILTER_TEXTS_OUT)?;
        Ok(())
    })
}

static S_RECONCILE_GROUP_FILTERS_0: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("id", wire::Schema::Num),
    ("source", wire::Schema::Str16),
    ("initial", wire::Schema::Str16),
    (
        "rows",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
            ("negate", wire::Schema::Bool),
            (
                "alternatives",
                wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
                    ("id", wire::Schema::Dyn),
                    ("kind", wire::Schema::Dyn),
                    ("value", wire::Schema::Dyn),
                ]))),
            ),
        ]))),
    ),
    ("advanced", wire::Schema::Bool),
]));
static S_RECONCILE_GROUP_FILTERS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("id", wire::Schema::Num),
    ("source", wire::Schema::Str16),
    ("initial", wire::Schema::Str16),
    (
        "rows",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
            ("negate", wire::Schema::Bool),
            (
                "alternatives",
                wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
                    ("id", wire::Schema::Dyn),
                    ("kind", wire::Schema::Dyn),
                    ("value", wire::Schema::Dyn),
                ]))),
            ),
        ]))),
    ),
    ("advanced", wire::Schema::Bool),
]));

/// `reconcileGroupFilters` over the binary wire.
#[wasm_bindgen(js_name = "reconcileGroupFiltersWire")]
pub fn reconcile_group_filters_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let previous: Vec<serde_json::Value> =
            r.list(|r| wire::read_value(r, S_RECONCILE_GROUP_FILTERS_0.inner()))?;
        let texts: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::reconcile_group_filters(previous, texts)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_RECONCILE_GROUP_FILTERS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_NEW_GROUP_FILTER_OUT: wire::Schema = wire::Schema::Obj(&[
    ("id", wire::Schema::Num),
    ("source", wire::Schema::Str16),
    ("initial", wire::Schema::Str16),
    (
        "rows",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
            ("negate", wire::Schema::Bool),
            (
                "alternatives",
                wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Obj(&[
                    ("id", wire::Schema::Num),
                    ("kind", wire::Schema::Dyn),
                    ("value", wire::Schema::Str16),
                ]))),
            ),
        ]))),
    ),
    ("advanced", wire::Schema::Bool),
]);

/// `newGroupFilter` over the binary wire.
#[wasm_bindgen(js_name = "newGroupFilterWire")]
pub fn new_group_filter_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |_r, w| {
        let out = &port::new_group_filter()?;
        wire::write_value(w, out, &S_NEW_GROUP_FILTER_OUT)?;
        Ok(())
    })
}

static S_GROUP_ADMITS_1: wire::Schema = wire::Schema::Dyn;
static S_GROUP_ADMITS_OUT: wire::Schema = wire::Schema::Dyn;

/// `groupAdmits` over the binary wire.
#[wasm_bindgen(js_name = "groupAdmitsWire")]
pub fn group_admits_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filters: Vec<String> = r.list(|r| r.string16())?;
        let node: serde_json::Value = wire::read_value(r, &S_GROUP_ADMITS_1)?;
        let out = &port::group_admits(filters, node)?;
        wire::write_value(w, out, &S_GROUP_ADMITS_OUT)?;
        Ok(())
    })
}

static S_READ_GROUP_ENTRIES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("name", wire::Schema::Str16),
    ("written", wire::Schema::Str16),
    ("filters", wire::Schema::List(&wire::Schema::Str16)),
    ("policy", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("default", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("final", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("exit", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("interrupt", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
]));

/// `readGroupEntries` over the binary wire.
#[wasm_bindgen(js_name = "readGroupEntriesWire")]
pub fn read_group_entries_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::read_group_entries(text)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_READ_GROUP_ENTRIES_OUT.inner())?;
        }
        Ok(())
    })
}

static S_QUOTE_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `quoteName` over the binary wire.
#[wasm_bindgen(js_name = "quoteNameWire")]
pub fn quote_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: String = r.string16()?;
        let out = &port::quote_name(value)?;
        wire::write_value(w, out, &S_QUOTE_NAME_OUT)?;
        Ok(())
    })
}

static S_IS_WRITABLE_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `isWritableName` over the binary wire.
#[wasm_bindgen(js_name = "isWritableNameWire")]
pub fn is_writable_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: String = r.string16()?;
        let out = &port::is_writable_name(value)?;
        wire::write_value(w, out, &S_IS_WRITABLE_NAME_OUT)?;
        Ok(())
    })
}

static S_GROUP_NAME_PROBLEM_1: wire::Schema = wire::Schema::Dyn;
static S_GROUP_NAME_PROBLEM_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `groupNameProblem` over the binary wire.
#[wasm_bindgen(js_name = "groupNameProblemWire")]
pub fn group_name_problem_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let name: String = r.string16()?;
        let taken: serde_json::Value = wire::read_value(r, &S_GROUP_NAME_PROBLEM_1)?;
        let out = &port::group_name_problem(name, taken)?;
        wire::write_value(w, out, &S_GROUP_NAME_PROBLEM_OUT)?;
        Ok(())
    })
}

static S_WRITE_GROUP_ENTRY_2: wire::Schema = wire::Schema::Dyn;

/// `writeGroupEntry` over the binary wire.
#[wasm_bindgen(js_name = "writeGroupEntryWire")]
pub fn write_group_entry_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let name: String = r.string16()?;
        let next: serde_json::Value = wire::read_value(r, &S_WRITE_GROUP_ENTRY_2)?;
        let out = &port::write_group_entry(text, name, next)?;
        w.string16(out);
        Ok(())
    })
}

static S_NAME_TEXT_0: wire::Schema = wire::Schema::Dyn;
static S_NAME_TEXT_1: wire::Schema = wire::Schema::Dyn;
static S_NAME_TEXT_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `nameText` over the binary wire.
#[wasm_bindgen(js_name = "nameTextWire")]
pub fn name_text_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: serde_json::Value = wire::read_value(r, &S_NAME_TEXT_0)?;
        let written: serde_json::Value = wire::read_value(r, &S_NAME_TEXT_1)?;
        let out = &port::name_text(value, written)?;
        wire::write_value(w, out, &S_NAME_TEXT_OUT)?;
        Ok(())
    })
}

static S_NESTED_IN_0: wire::Schema = wire::Schema::Dyn;

/// `nestedIn` over the binary wire.
#[wasm_bindgen(js_name = "nestedInWire")]
pub fn nested_in_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let entry: serde_json::Value = wire::read_value(r, &S_NESTED_IN_0)?;
        let out = &port::nested_in(entry)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            w.string16(x0);
        }
        Ok(())
    })
}

static S_EXACT_TOKENS_1: wire::Schema = wire::Schema::Dyn;
static S_EXACT_TOKENS_OUT: wire::Schema = wire::Schema::Dyn;

/// `exactTokens` over the binary wire.
#[wasm_bindgen(js_name = "exactTokensWire")]
pub fn exact_tokens_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filter: String = r.string16()?;
        let call: serde_json::Value = wire::read_value(r, &S_EXACT_TOKENS_1)?;
        let out = &port::exact_tokens(filter, call)?;
        wire::write_value(w, out, &S_EXACT_TOKENS_OUT)?;
        Ok(())
    })
}

static S_NAMED_IN_0: wire::Schema = wire::Schema::Dyn;

/// `namedIn` over the binary wire.
#[wasm_bindgen(js_name = "namedInWire")]
pub fn named_in_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let entry: serde_json::Value = wire::read_value(r, &S_NAMED_IN_0)?;
        let out = &port::named_in(entry)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            w.string16(x0);
        }
        Ok(())
    })
}

static S_CLASSIFY_FILTERS_0: wire::Schema = wire::Schema::Dyn;
static S_CLASSIFY_FILTERS_OUT: wire::Schema = wire::Schema::Dyn;

/// `classifyFilters` over the binary wire.
#[wasm_bindgen(js_name = "classifyFiltersWire")]
pub fn classify_filters_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let entry: serde_json::Value = wire::read_value(r, &S_CLASSIFY_FILTERS_0)?;
        let out = &port::classify_filters(entry)?;
        wire::write_value(w, out, &S_CLASSIFY_FILTERS_OUT)?;
        Ok(())
    })
}

static S_DESCRIBE_FILTERS_OUT: wire::Schema = wire::Schema::Dyn;

/// `describeFilters` over the binary wire.
#[wasm_bindgen(js_name = "describeFiltersWire")]
pub fn describe_filters_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let filters: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::describe_filters(filters)?;
        wire::write_value(w, out, &S_DESCRIBE_FILTERS_OUT)?;
        Ok(())
    })
}

static S_CITING_GROUPS_OUT: wire::Schema = wire::Schema::Dyn;

/// `citingGroups` over the binary wire.
#[wasm_bindgen(js_name = "citingGroupsWire")]
pub fn citing_groups_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let tag: String = r.string16()?;
        let out = &port::citing_groups(text, tag)?;
        wire::write_value(w, out, &S_CITING_GROUPS_OUT)?;
        Ok(())
    })
}

static S_NAMED_IN_EXPRESSION_OUT: wire::Schema = wire::Schema::Dyn;

/// `namedInExpression` over the binary wire.
#[wasm_bindgen(js_name = "namedInExpressionWire")]
pub fn named_in_expression_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let tag: String = r.string16()?;
        let out = &port::named_in_expression(text, tag)?;
        wire::write_value(w, out, &S_NAMED_IN_EXPRESSION_OUT)?;
        Ok(())
    })
}

static S_GROUPS_NAMING_TAG_OUT: wire::Schema = wire::Schema::Dyn;

/// `groupsNamingTag` over the binary wire.
#[wasm_bindgen(js_name = "groupsNamingTagWire")]
pub fn groups_naming_tag_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let tag: String = r.string16()?;
        let out = &port::groups_naming_tag(text, tag)?;
        wire::write_value(w, out, &S_GROUPS_NAMING_TAG_OUT)?;
        Ok(())
    })
}

static S_ADD_SUBTAGS_TO_GROUP_OUT: wire::Schema = wire::Schema::Dyn;

/// `addSubtagsToGroup` over the binary wire.
#[wasm_bindgen(js_name = "addSubtagsToGroupWire")]
pub fn add_subtags_to_group_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let group: String = r.string16()?;
        let tags: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::add_subtags_to_group(text, group, tags)?;
        wire::write_value(w, out, &S_ADD_SUBTAGS_TO_GROUP_OUT)?;
        Ok(())
    })
}

static S_ADD_NODES_TO_GROUP_OUT: wire::Schema = wire::Schema::Dyn;

/// `addNodesToGroup` over the binary wire.
#[wasm_bindgen(js_name = "addNodesToGroupWire")]
pub fn add_nodes_to_group_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let group: String = r.string16()?;
        let names: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::add_nodes_to_group(text, group, names)?;
        wire::write_value(w, out, &S_ADD_NODES_TO_GROUP_OUT)?;
        Ok(())
    })
}

static S_REMOVE_SUBTAGS_FROM_GROUP_OUT: wire::Schema = wire::Schema::Dyn;

/// `removeSubtagsFromGroup` over the binary wire.
#[wasm_bindgen(js_name = "removeSubtagsFromGroupWire")]
pub fn remove_subtags_from_group_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let group: String = r.string16()?;
        let tags: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::remove_subtags_from_group(text, group, tags)?;
        wire::write_value(w, out, &S_REMOVE_SUBTAGS_FROM_GROUP_OUT)?;
        Ok(())
    })
}

static S_RULE_CONDITION_0: wire::Schema = wire::Schema::Dyn;

/// `ruleCondition` over the binary wire.
#[wasm_bindgen(js_name = "ruleConditionWire")]
pub fn rule_condition_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let kind: serde_json::Value = wire::read_value(r, &S_RULE_CONDITION_0)?;
        let value: String = r.string16()?;
        let out = &port::rule_condition(kind, value)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_IS_GLOB_OUT: wire::Schema = wire::Schema::Dyn;

/// `isGlob` over the binary wire.
#[wasm_bindgen(js_name = "isGlobWire")]
pub fn is_glob_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let pattern: String = r.string16()?;
        let out = &port::is_glob(pattern)?;
        wire::write_value(w, out, &S_IS_GLOB_OUT)?;
        Ok(())
    })
}

static S_INCLUDED_BY_OUT: wire::Schema = wire::Schema::Dyn;

/// `includedBy` over the binary wire.
#[wasm_bindgen(js_name = "includedByWire")]
pub fn included_by_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let patterns: Vec<String> = r.list(|r| r.string16())?;
        let path: String = r.string16()?;
        let out = &port::included_by(patterns, path)?;
        wire::write_value(w, out, &S_INCLUDED_BY_OUT)?;
        Ok(())
    })
}

static S_RESOLVE_INCLUDE_PATH_OUT: wire::Schema = wire::Schema::Dyn;

/// `resolveIncludePath` over the binary wire.
#[wasm_bindgen(js_name = "resolveIncludePathWire")]
pub fn resolve_include_path_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let base: Option<String> = r.opt_js(|r| r.string16())?;
        let path: String = r.string16()?;
        let out = &port::resolve_include_path(base, path)?;
        wire::write_value(w, out, &S_RESOLVE_INCLUDE_PATH_OUT)?;
        Ok(())
    })
}

static S_NEW_SOURCE_PATH_PROBLEM_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `newSourcePathProblem` over the binary wire.
#[wasm_bindgen(js_name = "newSourcePathProblemWire")]
pub fn new_source_path_problem_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let path: String = r.string16()?;
        let out = &port::new_source_path_problem(path)?;
        wire::write_value(w, out, &S_NEW_SOURCE_PATH_PROBLEM_OUT)?;
        Ok(())
    })
}

static S_NAME_PATTERNS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("pattern", wire::Schema::Str16),
    ("prefix", wire::Schema::Str16),
    ("suffix", wire::Schema::Str16),
]));

/// `namePatterns` over the binary wire.
#[wasm_bindgen(js_name = "namePatternsWire")]
pub fn name_patterns_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let patterns: Vec<String> = r.list(|r| r.string16())?;
        let out = &port::name_patterns(patterns)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_NAME_PATTERNS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_NEW_SOURCE_NAME_PROBLEM_1: wire::Schema = wire::Schema::Dyn;
static S_NEW_SOURCE_NAME_PROBLEM_OUT: wire::Schema = wire::Schema::Dyn;

/// `newSourceNameProblem` over the binary wire.
#[wasm_bindgen(js_name = "newSourceNameProblemWire")]
pub fn new_source_name_problem_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let name: String = r.string16()?;
        let pattern: serde_json::Value = wire::read_value(r, &S_NEW_SOURCE_NAME_PROBLEM_1)?;
        let out = &port::new_source_name_problem(name, pattern)?;
        wire::write_value(w, out, &S_NEW_SOURCE_NAME_PROBLEM_OUT)?;
        Ok(())
    })
}

/// `sourceAt` over the binary wire.
#[wasm_bindgen(js_name = "sourceAtWire")]
pub fn source_at_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let sources: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let path: String = r.string16()?;
        let out = &port::source_at(sources, path)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_PROTOCOL_NAME_0: wire::Schema = wire::Schema::Dyn;
static S_PROTOCOL_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `protocolName` over the binary wire.
#[wasm_bindgen(js_name = "protocolNameWire")]
pub fn protocol_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let protocol: serde_json::Value = wire::read_value(r, &S_PROTOCOL_NAME_0)?;
        let out = &port::protocol_name(protocol)?;
        wire::write_value(w, out, &S_PROTOCOL_NAME_OUT)?;
        Ok(())
    })
}

static S_EMPTY_NODE_0: wire::Schema = wire::Schema::Dyn;
static S_EMPTY_NODE_OUT: wire::Schema = wire::Schema::Map16(&wire::Schema::Str16);

/// `emptyNode` over the binary wire.
#[wasm_bindgen(js_name = "emptyNodeWire")]
pub fn empty_node_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let protocol: serde_json::Value = wire::read_value(r, &S_EMPTY_NODE_0)?;
        let out = &port::empty_node(protocol)?;
        wire::write_value(w, out, &S_EMPTY_NODE_OUT)?;
        Ok(())
    })
}

static S_PARSE_NODE_LINK_OUT: wire::Schema = wire::Schema::Obj(&[
    (
        "document",
        wire::Schema::Obj(&[
            ("original", wire::Schema::Str16),
            ("initial", wire::Schema::Map16(&wire::Schema::Str16)),
            ("query", wire::Schema::Str16),
            (
                "json",
                wire::Schema::OptJs(&wire::Schema::Map16(&wire::Schema::Dyn)),
            ),
            ("tail", wire::Schema::Str16),
        ]),
    ),
    ("fields", wire::Schema::Map16(&wire::Schema::Str16)),
]);

/// `parseNodeLink` over the binary wire.
#[wasm_bindgen(js_name = "parseNodeLinkWire")]
pub fn parse_node_link_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let original: String = r.string16()?;
        let out = &port::parse_node_link(original)?;
        wire::write_value(w, out, &S_PARSE_NODE_LINK_OUT)?;
        Ok(())
    })
}

static S_NODE_LINK_0: wire::Schema = wire::Schema::Dyn;

/// `nodeLink` over the binary wire.
#[wasm_bindgen(js_name = "nodeLinkWire")]
pub fn node_link_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let fields: serde_json::Value = wire::read_value(r, &S_NODE_LINK_0)?;
        let document: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::node_link(fields, document)?;
        w.string16(out);
        Ok(())
    })
}

static S_SECURITY_OPTIONS_0: wire::Schema = wire::Schema::Dyn;
static S_SECURITY_OPTIONS_OUT: wire::Schema = wire::Schema::Dyn;

/// `securityOptions` over the binary wire.
#[wasm_bindgen(js_name = "securityOptionsWire")]
pub fn security_options_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let protocol: serde_json::Value = wire::read_value(r, &S_SECURITY_OPTIONS_0)?;
        let out = &port::security_options(protocol)?;
        wire::write_value(w, out, &S_SECURITY_OPTIONS_OUT)?;
        Ok(())
    })
}

static S_USES_TLS_0: wire::Schema = wire::Schema::Dyn;
static S_USES_TLS_OUT: wire::Schema = wire::Schema::Dyn;

/// `usesTls` over the binary wire.
#[wasm_bindgen(js_name = "usesTlsWire")]
pub fn uses_tls_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let f: serde_json::Value = wire::read_value(r, &S_USES_TLS_0)?;
        let out = &port::uses_tls(f)?;
        wire::write_value(w, out, &S_USES_TLS_OUT)?;
        Ok(())
    })
}

static S_MANUAL_FIELDS_0: wire::Schema = wire::Schema::Dyn;
static S_MANUAL_FIELDS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("key", wire::Schema::Str16),
    (
        "options",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Str16)),
    ),
    ("flag", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("required", wire::Schema::OptJs(&wire::Schema::Bool)),
    ("range", wire::Schema::Dyn),
]));

/// `manualFields` over the binary wire.
#[wasm_bindgen(js_name = "manualFieldsWire")]
pub fn manual_fields_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let f: serde_json::Value = wire::read_value(r, &S_MANUAL_FIELDS_0)?;
        let out = &port::manual_fields(f)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_MANUAL_FIELDS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_ADVANCED_FIELDS_0: wire::Schema = wire::Schema::Dyn;
static S_ADVANCED_FIELDS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("key", wire::Schema::Str16),
    (
        "options",
        wire::Schema::OptJs(&wire::Schema::List(&wire::Schema::Str16)),
    ),
    ("flag", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("required", wire::Schema::OptJs(&wire::Schema::Bool)),
    ("range", wire::Schema::Dyn),
]));

/// `advancedFields` over the binary wire.
#[wasm_bindgen(js_name = "advancedFieldsWire")]
pub fn advanced_fields_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let f: serde_json::Value = wire::read_value(r, &S_ADVANCED_FIELDS_0)?;
        let out = &port::advanced_fields(f)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_ADVANCED_FIELDS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_SETTLE_0: wire::Schema = wire::Schema::Dyn;
static S_SETTLE_OUT: wire::Schema = wire::Schema::Map16(&wire::Schema::Str16);

/// `settle` over the binary wire.
#[wasm_bindgen(js_name = "settleWire")]
pub fn settle_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let f: serde_json::Value = wire::read_value(r, &S_SETTLE_0)?;
        let out = &port::settle(f)?;
        wire::write_value(w, out, &S_SETTLE_OUT)?;
        Ok(())
    })
}

static S_MANUAL_INVALID_0: wire::Schema = wire::Schema::Dyn;

/// `manualInvalid` over the binary wire.
#[wasm_bindgen(js_name = "manualInvalidWire")]
pub fn manual_invalid_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let f: serde_json::Value = wire::read_value(r, &S_MANUAL_INVALID_0)?;
        let out = &port::manual_invalid(f)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            w.string16(x0);
        }
        Ok(())
    })
}

static S_MANUAL_LINK_0: wire::Schema = wire::Schema::Dyn;

/// `manualLink` over the binary wire.
#[wasm_bindgen(js_name = "manualLinkWire")]
pub fn manual_link_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let f: serde_json::Value = wire::read_value(r, &S_MANUAL_LINK_0)?;
        let out = &port::manual_link(f)?;
        w.string16(out);
        Ok(())
    })
}

static S_IS_NODE_LINK_OUT: wire::Schema = wire::Schema::Dyn;

/// `isNodeLink` over the binary wire.
#[wasm_bindgen(js_name = "isNodeLinkWire")]
pub fn is_node_link_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: String = r.string16()?;
        let out = &port::is_node_link(value)?;
        wire::write_value(w, out, &S_IS_NODE_LINK_OUT)?;
        Ok(())
    })
}

static S_READ_NODE_ENTRIES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("name", wire::Schema::Str16),
    ("link", wire::Schema::Str16),
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("line", wire::Schema::Num),
    ("nameTo", wire::Schema::Num),
    ("linkFrom", wire::Schema::Num),
]));

/// `readNodeEntries` over the binary wire.
#[wasm_bindgen(js_name = "readNodeEntriesWire")]
pub fn read_node_entries_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::read_node_entries(text)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_READ_NODE_ENTRIES_OUT.inner())?;
        }
        Ok(())
    })
}

static S_READ_CHAIN_ENTRIES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("name", wire::Schema::Str16),
    (
        "hops",
        wire::Schema::List(&wire::Schema::Obj(&[
            ("name", wire::Schema::Str16),
            ("from", wire::Schema::Num),
            ("to", wire::Schema::Num),
        ])),
    ),
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("line", wire::Schema::Num),
    ("nameTo", wire::Schema::Num),
    ("hopsFrom", wire::Schema::Num),
]));

/// `readChainEntries` over the binary wire.
#[wasm_bindgen(js_name = "readChainEntriesWire")]
pub fn read_chain_entries_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::read_chain_entries(text)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_READ_CHAIN_ENTRIES_OUT.inner())?;
        }
        Ok(())
    })
}

static S_NODE_REFERENCED_OUT: wire::Schema = wire::Schema::Dyn;

/// `nodeReferenced` over the binary wire.
#[wasm_bindgen(js_name = "nodeReferencedWire")]
pub fn node_referenced_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let name: String = r.string16()?;
        let out = &port::node_referenced(text, name)?;
        wire::write_value(w, out, &S_NODE_REFERENCED_OUT)?;
        Ok(())
    })
}

static S_NODE_REFERRERS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("kind", wire::Schema::Str16),
    ("name", wire::Schema::Str16),
]));

/// `nodeReferrers` over the binary wire.
#[wasm_bindgen(js_name = "nodeReferrersWire")]
pub fn node_referrers_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let name: String = r.string16()?;
        let out = &port::node_referrers(text, name)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_NODE_REFERRERS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_REMOVE_CHAIN_ENTRY_1: wire::Schema = wire::Schema::Dyn;

/// `removeChainEntry` over the binary wire.
#[wasm_bindgen(js_name = "removeChainEntryWire")]
pub fn remove_chain_entry_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let original: serde_json::Value = wire::read_value(r, &S_REMOVE_CHAIN_ENTRY_1)?;
        let out = &port::remove_chain_entry(text, original)?;
        w.string16(out);
        Ok(())
    })
}

static S_CHAIN_HOP_CANDIDATE_0: wire::Schema = wire::Schema::Dyn;
static S_CHAIN_HOP_CANDIDATE_OUT: wire::Schema = wire::Schema::Dyn;

/// `chainHopCandidate` over the binary wire.
#[wasm_bindgen(js_name = "chainHopCandidateWire")]
pub fn chain_hop_candidate_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let node: serde_json::Value = wire::read_value(r, &S_CHAIN_HOP_CANDIDATE_0)?;
        let out = &port::chain_hop_candidate(node)?;
        wire::write_value(w, out, &S_CHAIN_HOP_CANDIDATE_OUT)?;
        Ok(())
    })
}

static S_TUNNEL_DIALABLE_0: wire::Schema = wire::Schema::Dyn;
static S_TUNNEL_DIALABLE_OUT: wire::Schema = wire::Schema::Dyn;

/// `tunnelDialable` over the binary wire.
#[wasm_bindgen(js_name = "tunnelDialableWire")]
pub fn tunnel_dialable_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let node: serde_json::Value = wire::read_value(r, &S_TUNNEL_DIALABLE_0)?;
        let out = &port::tunnel_dialable(node)?;
        wire::write_value(w, out, &S_TUNNEL_DIALABLE_OUT)?;
        Ok(())
    })
}

/// `chainHopProblem` over the binary wire.
#[wasm_bindgen(js_name = "chainHopProblemWire")]
pub fn chain_hop_problem_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let nodes_list: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let name: String = r.string16()?;
        let index: f64 = r.f64()?;
        let out = &port::chain_hop_problem(nodes_list, name, index)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_CHAIN_UDP_PROBLEM_OUT: wire::Schema = wire::Schema::Dyn;

/// `chainUdpProblem` over the binary wire.
#[wasm_bindgen(js_name = "chainUdpProblemWire")]
pub fn chain_udp_problem_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let hops: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let out = &port::chain_udp_problem(hops)?;
        wire::write_value(w, out, &S_CHAIN_UDP_PROBLEM_OUT)?;
        Ok(())
    })
}

static S_WRITE_CHAIN_ENTRY_1: wire::Schema = wire::Schema::Dyn;
static S_WRITE_CHAIN_ENTRY_2: wire::Schema = wire::Schema::Dyn;

/// `writeChainEntry` over the binary wire.
#[wasm_bindgen(js_name = "writeChainEntryWire")]
pub fn write_chain_entry_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let original: serde_json::Value = wire::read_value(r, &S_WRITE_CHAIN_ENTRY_1)?;
        let next: serde_json::Value = wire::read_value(r, &S_WRITE_CHAIN_ENTRY_2)?;
        let out = &port::write_chain_entry(text, original, next)?;
        w.string16(out);
        Ok(())
    })
}

static S_APPEND_CHAIN_ENTRY_1: wire::Schema = wire::Schema::Dyn;

/// `appendChainEntry` over the binary wire.
#[wasm_bindgen(js_name = "appendChainEntryWire")]
pub fn append_chain_entry_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let next: serde_json::Value = wire::read_value(r, &S_APPEND_CHAIN_ENTRY_1)?;
        let out = &port::append_chain_entry(text, next)?;
        w.string16(out);
        Ok(())
    })
}

static S_WRITE_NODE_ENTRY_1: wire::Schema = wire::Schema::Dyn;
static S_WRITE_NODE_ENTRY_2: wire::Schema = wire::Schema::Dyn;

/// `writeNodeEntry` over the binary wire.
#[wasm_bindgen(js_name = "writeNodeEntryWire")]
pub fn write_node_entry_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let original: serde_json::Value = wire::read_value(r, &S_WRITE_NODE_ENTRY_1)?;
        let next: serde_json::Value = wire::read_value(r, &S_WRITE_NODE_ENTRY_2)?;
        let out = &port::write_node_entry(text, original, next)?;
        w.string16(out);
        Ok(())
    })
}

static S_READ_MODE_OUT: wire::Schema = wire::Schema::Dyn;

/// `readMode` over the binary wire.
#[wasm_bindgen(js_name = "readModeWire")]
pub fn read_mode_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::read_mode(text)?;
        wire::write_value(w, out, &S_READ_MODE_OUT)?;
        Ok(())
    })
}

static S_WRITE_MODE_1: wire::Schema = wire::Schema::Dyn;

/// `writeMode` over the binary wire.
#[wasm_bindgen(js_name = "writeModeWire")]
pub fn write_mode_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let next: serde_json::Value = wire::read_value(r, &S_WRITE_MODE_1)?;
        let out = &port::write_mode(text, next)?;
        w.string16(out);
        Ok(())
    })
}

static S_SAME_MODE_0: wire::Schema = wire::Schema::Dyn;
static S_SAME_MODE_1: wire::Schema = wire::Schema::Dyn;
static S_SAME_MODE_OUT: wire::Schema = wire::Schema::Dyn;

/// `sameMode` over the binary wire.
#[wasm_bindgen(js_name = "sameModeWire")]
pub fn same_mode_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let a: serde_json::Value = wire::read_value(r, &S_SAME_MODE_0)?;
        let b: serde_json::Value = wire::read_value(r, &S_SAME_MODE_1)?;
        let out = &port::same_mode(a, b)?;
        wire::write_value(w, out, &S_SAME_MODE_OUT)?;
        Ok(())
    })
}

static S_MODE_RULE_LINES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Num);

/// `modeRuleLines` over the binary wire.
#[wasm_bindgen(js_name = "modeRuleLinesWire")]
pub fn mode_rule_lines_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::mode_rule_lines(text)?;
        wire::write_value(w, out, &S_MODE_RULE_LINES_OUT)?;
        Ok(())
    })
}

static S_NODE_PROBE_SUPPORT_OUT: wire::Schema = wire::Schema::Dyn;

/// `nodeProbeSupport` over the binary wire.
#[wasm_bindgen(js_name = "nodeProbeSupportWire")]
pub fn node_probe_support_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let protocol: Option<String> = r.opt_js(|r| r.string16())?;
        let out = &port::node_probe_support(protocol)?;
        wire::write_value(w, out, &S_NODE_PROBE_SUPPORT_OUT)?;
        Ok(())
    })
}

static S_SUPPORTS_NODE_PROBE_1: wire::Schema = wire::Schema::Dyn;
static S_SUPPORTS_NODE_PROBE_2: wire::Schema = wire::Schema::Dyn;

/// `supportsNodeProbe` over the binary wire.
#[wasm_bindgen(js_name = "supportsNodeProbeWire")]
pub fn supports_node_probe_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let protocol: Option<String> = r.opt_js(|r| r.string16())?;
        let kind: serde_json::Value = wire::read_value(r, &S_SUPPORTS_NODE_PROBE_1)?;
        let transports: serde_json::Value = wire::read_value(r, &S_SUPPORTS_NODE_PROBE_2)?;
        let out = &port::supports_node_probe(protocol, kind, transports)?;
        w.bool(*out);
        Ok(())
    })
}

static S_REGION_FLAG_OUT: wire::Schema = wire::Schema::Dyn;

/// `regionFlag` over the binary wire.
#[wasm_bindgen(js_name = "regionFlagWire")]
pub fn region_flag_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let region: String = r.string16()?;
        let out = &port::region_flag(region)?;
        wire::write_value(w, out, &S_REGION_FLAG_OUT)?;
        Ok(())
    })
}

static S_MATCH_REGION_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `matchRegion` over the binary wire.
#[wasm_bindgen(js_name = "matchRegionWire")]
pub fn match_region_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let name: String = r.string16()?;
        let out = &port::match_region(name)?;
        wire::write_value(w, out, &S_MATCH_REGION_OUT)?;
        Ok(())
    })
}

static S_SOURCE_FOR_0: wire::Schema = wire::Schema::Dyn;
static S_SOURCE_FOR_1: wire::Schema = wire::Schema::Dyn;
static S_SOURCE_FOR_OUT: wire::Schema = wire::Schema::Dyn;

/// `sourceFor` over the binary wire.
#[wasm_bindgen(js_name = "sourceForWire")]
pub fn source_for_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let list: serde_json::Value = wire::read_value(r, &S_SOURCE_FOR_0)?;
        let source: serde_json::Value = wire::read_value(r, &S_SOURCE_FOR_1)?;
        let out = &port::source_for(list, source)?;
        wire::write_value(w, out, &S_SOURCE_FOR_OUT)?;
        Ok(())
    })
}

static S_RULE_ANCHOR_0: wire::Schema = wire::Schema::Dyn;
static S_RULE_ANCHOR_1: wire::Schema = wire::Schema::Dyn;
static S_RULE_ANCHOR_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Obj(&[
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("indent", wire::Schema::Str16),
    ("text", wire::Schema::Str16),
    (
        "target",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("from", wire::Schema::Num),
            ("to", wire::Schema::Num),
            ("foldCase", wire::Schema::OptJs(&wire::Schema::Bool)),
        ])),
    ),
    (
        "condition",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("from", wire::Schema::Num),
            ("to", wire::Schema::Num),
        ])),
    ),
    ("open", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("close", wire::Schema::OptJs(&wire::Schema::Str16)),
]));

/// `ruleAnchor` over the binary wire.
#[wasm_bindgen(js_name = "ruleAnchorWire")]
pub fn rule_anchor_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let source: serde_json::Value = wire::read_value(r, &S_RULE_ANCHOR_0)?;
        let rule: serde_json::Value = wire::read_value(r, &S_RULE_ANCHOR_1)?;
        let scan: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::rule_anchor(source, rule, scan)?;
        match out {
            Some(x0) => {
                w.tag(1);
                wire::write_value(w, x0, S_RULE_ANCHOR_OUT.inner())?;
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_DNS_RULE_TARGET_0: wire::Schema = wire::Schema::Dyn;
static S_DNS_RULE_TARGET_OUT: wire::Schema = wire::Schema::Dyn;

/// `dnsRuleTarget` over the binary wire.
#[wasm_bindgen(js_name = "dnsRuleTargetWire")]
pub fn dns_rule_target_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let rule: serde_json::Value = wire::read_value(r, &S_DNS_RULE_TARGET_0)?;
        let out = &port::dns_rule_target(rule)?;
        wire::write_value(w, out, &S_DNS_RULE_TARGET_OUT)?;
        Ok(())
    })
}

static S_DNS_RULE_ANCHOR_0: wire::Schema = wire::Schema::Dyn;
static S_DNS_RULE_ANCHOR_1: wire::Schema = wire::Schema::Dyn;
static S_DNS_RULE_ANCHOR_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Obj(&[
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("indent", wire::Schema::Str16),
    ("text", wire::Schema::Str16),
    (
        "target",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("from", wire::Schema::Num),
            ("to", wire::Schema::Num),
            ("foldCase", wire::Schema::OptJs(&wire::Schema::Bool)),
        ])),
    ),
    (
        "condition",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("from", wire::Schema::Num),
            ("to", wire::Schema::Num),
        ])),
    ),
    ("open", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("close", wire::Schema::OptJs(&wire::Schema::Str16)),
]));

/// `dnsRuleAnchor` over the binary wire.
#[wasm_bindgen(js_name = "dnsRuleAnchorWire")]
pub fn dns_rule_anchor_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let source: serde_json::Value = wire::read_value(r, &S_DNS_RULE_ANCHOR_0)?;
        let rule: serde_json::Value = wire::read_value(r, &S_DNS_RULE_ANCHOR_1)?;
        let list: String = r.string16()?;
        let scan: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::dns_rule_anchor(source, rule, list, scan)?;
        match out {
            Some(x0) => {
                w.tag(1);
                wire::write_value(w, x0, S_DNS_RULE_ANCHOR_OUT.inner())?;
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

/// `dnsListEnd` over the binary wire.
#[wasm_bindgen(js_name = "dnsListEndWire")]
pub fn dns_list_end_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let sources: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let list: String = r.string16()?;
        let out = &port::dns_list_end(sources, list)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_DNS_UPSTREAMS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("name", wire::Schema::Str16),
    ("address", wire::Schema::Str16),
]));

/// `dnsUpstreams` over the binary wire.
#[wasm_bindgen(js_name = "dnsUpstreamsWire")]
pub fn dns_upstreams_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let scan: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::dns_upstreams(text, scan)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_DNS_UPSTREAMS_OUT.inner())?;
        }
        Ok(())
    })
}

/// `dnsUpstreamNames` over the binary wire.
#[wasm_bindgen(js_name = "dnsUpstreamNamesWire")]
pub fn dns_upstream_names_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let scan: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::dns_upstream_names(text, scan)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            w.string16(x0);
        }
        Ok(())
    })
}

static S_RULE_LINE_OUT: wire::Schema = wire::Schema::Dyn;

/// `ruleLine` over the binary wire.
#[wasm_bindgen(js_name = "ruleLineWire")]
pub fn rule_line_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let condition: String = r.string16()?;
        let outbound: String = r.string16()?;
        let must: Option<bool> = r.opt_js(|r| r.bool())?;
        let out = &port::rule_line(condition, outbound, must)?;
        wire::write_value(w, out, &S_RULE_LINE_OUT)?;
        Ok(())
    })
}

static S_ADD_RULE_1: wire::Schema = wire::Schema::Dyn;

/// `addRule` over the binary wire.
#[wasm_bindgen(js_name = "addRuleWire")]
pub fn add_rule_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let anchor: serde_json::Value = wire::read_value(r, &S_ADD_RULE_1)?;
        let condition: String = r.string16()?;
        let outbound: String = r.string16()?;
        let must: bool = r.bool()?;
        let out = &port::add_rule(text, anchor, condition, outbound, must)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_REPLACE_RULE_TARGET_1: wire::Schema = wire::Schema::Dyn;

/// `replaceRuleTarget` over the binary wire.
#[wasm_bindgen(js_name = "replaceRuleTargetWire")]
pub fn replace_rule_target_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let anchor: serde_json::Value = wire::read_value(r, &S_REPLACE_RULE_TARGET_1)?;
        let outbound: String = r.string16()?;
        let must: bool = r.bool()?;
        let out = &port::replace_rule_target(text, anchor, outbound, must)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_REMOVE_RULE_1: wire::Schema = wire::Schema::Dyn;

/// `removeRule` over the binary wire.
#[wasm_bindgen(js_name = "removeRuleWire")]
pub fn remove_rule_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let anchor: serde_json::Value = wire::read_value(r, &S_REMOVE_RULE_1)?;
        let out = &port::remove_rule(text, anchor)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_REPLACE_RULE_1: wire::Schema = wire::Schema::Dyn;

/// `replaceRule` over the binary wire.
#[wasm_bindgen(js_name = "replaceRuleWire")]
pub fn replace_rule_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let anchor: serde_json::Value = wire::read_value(r, &S_REPLACE_RULE_1)?;
        let condition: String = r.string16()?;
        let outbound: String = r.string16()?;
        let must: bool = r.bool()?;
        let out = &port::replace_rule(text, anchor, condition, outbound, must)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_ADD_FALLBACK_1: wire::Schema = wire::Schema::Dyn;

/// `addFallback` over the binary wire.
#[wasm_bindgen(js_name = "addFallbackWire")]
pub fn add_fallback_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let anchor: serde_json::Value = wire::read_value(r, &S_ADD_FALLBACK_1)?;
        let target: String = r.string16()?;
        let out = &port::add_fallback(text, anchor, target)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_NUMBER_SETTING_0: wire::Schema = wire::Schema::Dyn;

/// `numberSetting` over the binary wire.
#[wasm_bindgen(js_name = "numberSettingWire")]
pub fn number_setting_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let field: serde_json::Value = wire::read_value(r, &S_NUMBER_SETTING_0)?;
        let out = &port::number_setting(field)?;
        w.bool(*out);
        Ok(())
    })
}

static S_SETTING_VALUE_0: wire::Schema = wire::Schema::Dyn;

/// `settingValue` over the binary wire.
#[wasm_bindgen(js_name = "settingValueWire")]
pub fn setting_value_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let field: serde_json::Value = wire::read_value(r, &S_SETTING_VALUE_0)?;
        let value: String = r.string16()?;
        let out = &port::setting_value(field, value)?;
        w.string16(out);
        Ok(())
    })
}

static S_SERIALIZE_SETTING_0: wire::Schema = wire::Schema::Dyn;

/// `serializeSetting` over the binary wire.
#[wasm_bindgen(js_name = "serializeSettingWire")]
pub fn serialize_setting_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let field: serde_json::Value = wire::read_value(r, &S_SERIALIZE_SETTING_0)?;
        let value: String = r.string16()?;
        let out = &port::serialize_setting(field, value)?;
        match out {
            Some(x0) => {
                w.tag(1);
                w.string16(x0);
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_WRITE_SETTINGS_1: wire::Schema = wire::Schema::Dyn;
static S_WRITE_SETTINGS_3: wire::Schema = wire::Schema::Dyn;

/// `writeSettings` over the binary wire.
#[wasm_bindgen(js_name = "writeSettingsWire")]
pub fn write_settings_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let section: serde_json::Value = wire::read_value(r, &S_WRITE_SETTINGS_1)?;
        let index: f64 = r.f64()?;
        let patch: serde_json::Value = wire::read_value(r, &S_WRITE_SETTINGS_3)?;
        let out = &port::write_settings(text, section, index, patch)?;
        w.string16(out);
        Ok(())
    })
}

static S_ALL_GROUP_NAMES_OUT: wire::Schema = wire::Schema::Dyn;

/// `allGroupNames` over the binary wire.
#[wasm_bindgen(js_name = "allGroupNamesWire")]
pub fn all_group_names_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let sources: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let draft: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::all_group_names(sources, draft)?;
        wire::write_value(w, out, &S_ALL_GROUP_NAMES_OUT)?;
        Ok(())
    })
}

static S_ALL_UPSTREAM_NAMES_OUT: wire::Schema = wire::Schema::Dyn;

/// `allUpstreamNames` over the binary wire.
#[wasm_bindgen(js_name = "allUpstreamNamesWire")]
pub fn all_upstream_names_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let sources: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let draft: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::all_upstream_names(sources, draft)?;
        wire::write_value(w, out, &S_ALL_UPSTREAM_NAMES_OUT)?;
        Ok(())
    })
}

static S_REDACTED_0: wire::Schema = wire::Schema::Dyn;
static S_REDACTED_OUT: wire::Schema = wire::Schema::Dyn;

/// `redacted` over the binary wire.
#[wasm_bindgen(js_name = "redactedWire")]
pub fn redacted_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let source: serde_json::Value = wire::read_value(r, &S_REDACTED_0)?;
        let out = &port::redacted(source)?;
        wire::write_value(w, out, &S_REDACTED_OUT)?;
        Ok(())
    })
}

static S_FILE_NAME_0: wire::Schema = wire::Schema::Dyn;
static S_FILE_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `fileName` over the binary wire.
#[wasm_bindgen(js_name = "fileNameWire")]
pub fn file_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let source: serde_json::Value = wire::read_value(r, &S_FILE_NAME_0)?;
        let out = &port::file_name(source)?;
        wire::write_value(w, out, &S_FILE_NAME_OUT)?;
        Ok(())
    })
}

/// `validationSources` over the binary wire.
#[wasm_bindgen(js_name = "validationSourcesWire")]
pub fn validation_sources_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let sources: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let replacement: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::validation_sources(sources, replacement)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_RESTART_SETTINGS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("key", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("sourceId", wire::Schema::Str16),
    ("line", wire::Schema::OptJs(&wire::Schema::Num)),
]));

/// `restartSettings` over the binary wire.
#[wasm_bindgen(js_name = "restartSettingsWire")]
pub fn restart_settings_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let diagnostics: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let out = &port::restart_settings(diagnostics)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_RESTART_SETTINGS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_RESTART_REQUIRED_OUT: wire::Schema = wire::Schema::Dyn;

/// `restartRequired` over the binary wire.
#[wasm_bindgen(js_name = "restartRequiredWire")]
pub fn restart_required_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let diagnostics: Vec<serde_json::Value> =
            serde_json::from_value(wire::read_dyn(r)?).map_err(|e| format!("TypeError: {e}"))?;
        let out = &port::restart_required(diagnostics)?;
        wire::write_value(w, out, &S_RESTART_REQUIRED_OUT)?;
        Ok(())
    })
}

static S_UNWRITTEN_RESTART_SETTINGS_0: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("key", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("sourceId", wire::Schema::Str16),
    ("line", wire::Schema::OptJs(&wire::Schema::Num)),
]));
static S_UNWRITTEN_RESTART_SETTINGS_1: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[(
    "content",
    wire::Schema::OptJs(&wire::Schema::Str16),
)]));
static S_UNWRITTEN_RESTART_SETTINGS_OUT: wire::Schema = wire::Schema::Dyn;

/// `unwrittenRestartSettings` over the binary wire.
#[wasm_bindgen(js_name = "unwrittenRestartSettingsWire")]
pub fn unwritten_restart_settings_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let settings_list: Vec<serde_json::Value> =
            r.list(|r| wire::read_value(r, S_UNWRITTEN_RESTART_SETTINGS_0.inner()))?;
        let sources: Vec<serde_json::Value> =
            r.list(|r| wire::read_value(r, S_UNWRITTEN_RESTART_SETTINGS_1.inner()))?;
        let out = &port::unwritten_restart_settings(settings_list, sources)?;
        wire::write_value(w, out, &S_UNWRITTEN_RESTART_SETTINGS_OUT)?;
        Ok(())
    })
}

static S_COMPLETE_SUBSCRIPTION_URL_OUT: wire::Schema = wire::Schema::Dyn;

/// `completeSubscriptionUrl` over the binary wire.
#[wasm_bindgen(js_name = "completeSubscriptionUrlWire")]
pub fn complete_subscription_url_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: String = r.string16()?;
        let out = &port::complete_subscription_url(value)?;
        wire::write_value(w, out, &S_COMPLETE_SUBSCRIPTION_URL_OUT)?;
        Ok(())
    })
}

static S_IS_SUBSCRIPTION_URL_OUT: wire::Schema = wire::Schema::Dyn;

/// `isSubscriptionUrl` over the binary wire.
#[wasm_bindgen(js_name = "isSubscriptionUrlWire")]
pub fn is_subscription_url_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: String = r.string16()?;
        let out = &port::is_subscription_url(value)?;
        wire::write_value(w, out, &S_IS_SUBSCRIPTION_URL_OUT)?;
        Ok(())
    })
}

static S_PARSE_INTERVAL_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Num);

/// `parseInterval` over the binary wire.
#[wasm_bindgen(js_name = "parseIntervalWire")]
pub fn parse_interval_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: String = r.string16()?;
        let out = &port::parse_interval(value)?;
        wire::write_value(w, out, &S_PARSE_INTERVAL_OUT)?;
        Ok(())
    })
}

static S_URL_HOST_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `urlHost` over the binary wire.
#[wasm_bindgen(js_name = "urlHostWire")]
pub fn url_host_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let url: Option<String> = r.opt_js(|r| r.string16())?;
        let out = &port::url_host(url)?;
        wire::write_value(w, out, &S_URL_HOST_OUT)?;
        Ok(())
    })
}

/// `agentProblem` over the binary wire.
#[wasm_bindgen(js_name = "agentProblemWire")]
pub fn agent_problem_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let agent: String = r.string16()?;
        let written: bool = r.bool()?;
        let out = &port::agent_problem(agent, written)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_READ_SUBSCRIPTION_ENTRIES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("tag", wire::Schema::Str16),
    ("url", wire::Schema::Str16),
    ("form", wire::Schema::Str16),
    ("naming", wire::Schema::Str16),
    ("ua", wire::Schema::OptJs(&wire::Schema::Str16)),
    ("interval", wire::Schema::OptJs(&wire::Schema::Num)),
    ("cache", wire::Schema::OptJs(&wire::Schema::Bool)),
    ("route", wire::Schema::OptJs(&wire::Schema::Str16)),
    (
        "options",
        wire::Schema::List(&wire::Schema::Obj(&[
            ("name", wire::Schema::Str16),
            ("value", wire::Schema::Str16),
        ])),
    ),
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("line", wire::Schema::Num),
    (
        "tagAt",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("from", wire::Schema::Num),
            ("to", wire::Schema::Num),
        ])),
    ),
    (
        "urlAt",
        wire::Schema::Obj(&[("from", wire::Schema::Num), ("to", wire::Schema::Num)]),
    ),
]));

/// `readSubscriptionEntries` over the binary wire.
#[wasm_bindgen(js_name = "readSubscriptionEntriesWire")]
pub fn read_subscription_entries_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::read_subscription_entries(text)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_READ_SUBSCRIPTION_ENTRIES_OUT.inner())?;
        }
        Ok(())
    })
}

static S_WRITE_SUBSCRIPTION_ENTRY_2: wire::Schema = wire::Schema::Dyn;

/// `writeSubscriptionEntry` over the binary wire.
#[wasm_bindgen(js_name = "writeSubscriptionEntryWire")]
pub fn write_subscription_entry_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let tag: String = r.string16()?;
        let change: serde_json::Value = wire::read_value(r, &S_WRITE_SUBSCRIPTION_ENTRY_2)?;
        let out = &port::write_subscription_entry(text, tag, change)?;
        w.string16(out);
        Ok(())
    })
}

/// `templateRules` over the binary wire.
#[wasm_bindgen(js_name = "templateRulesWire")]
pub fn template_rules_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let template: String = r.string16()?;
        let options: Option<serde_json::Value> = wire::read_dyn_opt(r)?;
        let out = &port::template_rules(template, options)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            w.string16(x0);
        }
        Ok(())
    })
}

static S_DETECT_TEMPLATE_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Obj(&[
    ("blockAds", wire::Schema::Bool),
    ("blockQuic", wire::Schema::Bool),
    ("networkManagerDirect", wire::Schema::Bool),
    ("template", wire::Schema::Str16),
    ("group", wire::Schema::OptJs(&wire::Schema::Str16)),
]));

/// `detectTemplate` over the binary wire.
#[wasm_bindgen(js_name = "detectTemplateWire")]
pub fn detect_template_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::detect_template(text)?;
        match out {
            Some(x0) => {
                w.tag(1);
                wire::write_value(w, x0, S_DETECT_TEMPLATE_OUT.inner())?;
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_ROUTING_OPTIONS_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Obj(&[
    ("blockAds", wire::Schema::Bool),
    ("blockQuic", wire::Schema::Bool),
    ("networkManagerDirect", wire::Schema::Bool),
]));

/// `routingOptions` over the binary wire.
#[wasm_bindgen(js_name = "routingOptionsWire")]
pub fn routing_options_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::routing_options(text)?;
        match out {
            Some(x0) => {
                w.tag(1);
                wire::write_value(w, x0, S_ROUTING_OPTIONS_OUT.inner())?;
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_IS_BARE_NAME_OUT: wire::Schema = wire::Schema::Dyn;

/// `isBareName` over the binary wire.
#[wasm_bindgen(js_name = "isBareNameWire")]
pub fn is_bare_name_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let value: String = r.string16()?;
        let out = &port::is_bare_name(value)?;
        wire::write_value(w, out, &S_IS_BARE_NAME_OUT)?;
        Ok(())
    })
}

static S_SCAN_CONFIG_OUT: wire::Schema = wire::Schema::Dyn;

/// `scanConfig` over the binary wire.
#[wasm_bindgen(js_name = "scanConfigWire")]
pub fn scan_config_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::scan_config(text)?;
        wire::write_value(w, out, &S_SCAN_CONFIG_OUT)?;
        Ok(())
    })
}

static S_UNQUOTE_OUT: wire::Schema = wire::Schema::Dyn;

/// `unquote` over the binary wire.
#[wasm_bindgen(js_name = "unquoteWire")]
pub fn unquote_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let out = &port::unquote(text)?;
        wire::write_value(w, out, &S_UNQUOTE_OUT)?;
        Ok(())
    })
}

static S_BLOCK_FIELDS_1: wire::Schema = wire::Schema::Dyn;
static S_BLOCK_FIELDS_2: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("line", wire::Schema::Num),
    ("kind", wire::Schema::Str16),
    ("depth", wire::Schema::Num),
    ("parens", wire::Schema::Num),
]));
static S_BLOCK_FIELDS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("name", wire::Schema::Str16),
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    ("valueFrom", wire::Schema::Num),
    ("valueTo", wire::Schema::Num),
    ("value", wire::Schema::Str16),
]));

/// `blockFields` over the binary wire.
#[wasm_bindgen(js_name = "blockFieldsWire")]
pub fn block_fields_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let block: serde_json::Value = wire::read_value(r, &S_BLOCK_FIELDS_1)?;
        let tokens: Vec<serde_json::Value> =
            r.list(|r| wire::read_value(r, S_BLOCK_FIELDS_2.inner()))?;
        let out = &port::block_fields(text, block, tokens)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_BLOCK_FIELDS_OUT.inner())?;
        }
        Ok(())
    })
}

static S_BLOCK_BODY_1: wire::Schema = wire::Schema::Dyn;

/// `blockBody` over the binary wire.
#[wasm_bindgen(js_name = "blockBodyWire")]
pub fn block_body_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let block: serde_json::Value = wire::read_value(r, &S_BLOCK_BODY_1)?;
        let out = &port::block_body(text, block)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            w.string16(x0);
        }
        Ok(())
    })
}

static S_BLOCK_ENTRIES_1: wire::Schema = wire::Schema::Dyn;
static S_BLOCK_ENTRIES_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Obj(&[
    ("from", wire::Schema::Num),
    ("to", wire::Schema::Num),
    (
        "block",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("name", wire::Schema::Str16),
            ("from", wire::Schema::Num),
            ("open", wire::Schema::Num),
            ("close", wire::Schema::Num),
            ("to", wire::Schema::Num),
            ("line", wire::Schema::Num),
            ("endLine", wire::Schema::Num),
            ("depth", wire::Schema::Num),
            (
                "children",
                wire::Schema::List(&wire::Schema::Obj(&[
                    ("name", wire::Schema::Str16),
                    ("from", wire::Schema::Num),
                    ("open", wire::Schema::Num),
                    ("close", wire::Schema::Num),
                    ("to", wire::Schema::Num),
                    ("line", wire::Schema::Num),
                    ("endLine", wire::Schema::Num),
                    ("depth", wire::Schema::Num),
                    ("children", wire::Schema::List(&wire::Schema::Dyn)),
                ])),
            ),
        ])),
    ),
]));

/// `blockEntries` over the binary wire.
#[wasm_bindgen(js_name = "blockEntriesWire")]
pub fn block_entries_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let text: String = r.string16()?;
        let block: serde_json::Value = wire::read_value(r, &S_BLOCK_ENTRIES_1)?;
        let out = &port::block_entries(text, block)?;
        w.tag(1);
        w.count(out.len());
        for x0 in out.iter() {
            wire::write_value(w, x0, S_BLOCK_ENTRIES_OUT.inner())?;
        }
        Ok(())
    })
}

/// `policyKind` over the binary wire.
#[wasm_bindgen(js_name = "policyKindWire")]
pub fn policy_kind_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let native: String = r.string16()?;
        let out = &port::policy_kind(native)?;
        wire::write_dyn(w, &serde_json::to_value(out).map_err(|e| e.to_string())?);
        Ok(())
    })
}

static S_READ_XHTTP_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Obj(&[
    ("mode", wire::Schema::Str16),
    ("path", wire::Schema::Str16),
    ("host", wire::Schema::Str16),
    ("headers", wire::Schema::List(&wire::Schema::Dyn)),
    ("xPaddingBytes", wire::Schema::Str16),
    ("noGRPCHeader", wire::Schema::Bool),
    ("scMaxEachPostBytes", wire::Schema::Str16),
    ("scMinPostsIntervalMs", wire::Schema::Str16),
    (
        "download",
        wire::Schema::OptJs(&wire::Schema::Obj(&[
            ("address", wire::Schema::Str16),
            ("port", wire::Schema::Str16),
            ("serverName", wire::Schema::Str16),
            ("path", wire::Schema::Str16),
            ("host", wire::Schema::Str16),
        ])),
    ),
]));

/// `readXhttp` over the binary wire.
#[wasm_bindgen(js_name = "readXhttpWire")]
pub fn read_xhttp_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let link: String = r.string16()?;
        let out = &port::read_xhttp(link)?;
        match out {
            Some(x0) => {
                w.tag(1);
                wire::write_value(w, x0, S_READ_XHTTP_OUT.inner())?;
            }
            None => w.tag(2),
        }
        Ok(())
    })
}

static S_DUPLICATE_XHTTP_HEADERS_0: wire::Schema = wire::Schema::Dyn;
static S_DUPLICATE_XHTTP_HEADERS_OUT: wire::Schema = wire::Schema::List(&wire::Schema::Num);

/// `duplicateXhttpHeaders` over the binary wire.
#[wasm_bindgen(js_name = "duplicateXhttpHeadersWire")]
pub fn duplicate_xhttp_headers_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let headers: serde_json::Value = wire::read_value(r, &S_DUPLICATE_XHTTP_HEADERS_0)?;
        let out = &port::duplicate_xhttp_headers(headers)?;
        wire::write_value(w, out, &S_DUPLICATE_XHTTP_HEADERS_OUT)?;
        Ok(())
    })
}

static S_WRITE_XHTTP_1: wire::Schema = wire::Schema::Dyn;

/// `writeXhttp` over the binary wire.
#[wasm_bindgen(js_name = "writeXhttpWire")]
pub fn write_xhttp_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let link: String = r.string16()?;
        let change: serde_json::Value = wire::read_value(r, &S_WRITE_XHTTP_1)?;
        let out = &port::write_xhttp(link, change)?;
        w.string16(out);
        Ok(())
    })
}
