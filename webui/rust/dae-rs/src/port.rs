//! The Rust port of `src/dae/`.
//!
//! Organizes the Doona configuration, routing, node, group, subscription, and template
//! engine into focused submodules under `src/port/` and exposes all 141 exported functions.

#![forbid(unsafe_code)]

mod conditions;
mod geodata;
mod groups;
mod js_str;
mod new_source;
mod node_form;
mod nodes;
mod outbound_mode;
mod regions;
mod rule_text;
mod settings;
mod subscriptions;
mod templates;
mod text;
mod vocab;
mod xhttp;

/// Port of `serializeConditions` (`src/dae/conditions.ts:6`).
pub fn serialize_conditions(rows: serde_json::Value) -> Result<Option<String>, String> {
    conditions::serialize_conditions(rows)
}

/// Port of `parseConditions` (`src/dae/conditions.ts:13`).
pub fn parse_conditions(
    text: String,
    kinds: Vec<String>,
) -> Result<Option<Vec<serde_json::Value>>, String> {
    conditions::parse_conditions(text, kinds)
}

/// Port of `flagKey` (`src/dae/flags.ts:6`).
pub fn flag_key(name: String) -> Result<serde_json::Value, String> {
    regions::flag_key(name)
}

/// Port of `validFlagName` (`src/dae/flags.ts:7`).
pub fn valid_flag_name(name: String) -> Result<serde_json::Value, String> {
    regions::valid_flag_name(name)
}

/// Port of `hasEmbeddedFlag` (`src/dae/flags.ts:8`).
pub fn has_embedded_flag(name: String) -> Result<serde_json::Value, String> {
    regions::has_embedded_flag(name)
}

/// Port of `validFlagChoice` (`src/dae/flags.ts:9`).
pub fn valid_flag_choice(value: serde_json::Value) -> Result<bool, String> {
    regions::valid_flag_choice(value)
}

/// Port of `flagChoices` (`src/dae/flags.ts:15`).
pub fn flag_choices(locale: String) -> Result<Vec<serde_json::Value>, String> {
    regions::flag_choices(locale)
}

/// Port of `matchPreset` (`src/dae/geodata.ts:20`).
pub fn match_preset(urls: serde_json::Value) -> Result<Option<serde_json::Value>, String> {
    geodata::match_preset(urls)
}

/// Port of `missingCategories` (`src/dae/geodata.ts:29`).
pub fn missing_categories(
    preset: Option<serde_json::Value>,
    required: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>, String> {
    geodata::missing_categories(preset, required)
}

/// Port of `geodataPreset` (`src/dae/geodata.ts:103`).
pub fn geodata_preset(id: String) -> Result<serde_json::Value, String> {
    geodata::geodata_preset(id)
}

/// Port of `validGeodataUrl` (`src/dae/geodata.ts:109`).
pub fn valid_geodata_url(url: String) -> Result<bool, String> {
    geodata::valid_geodata_url(url)
}

/// Port of `conditionFamily` (`src/dae/groupConditions.ts:7`).
pub fn condition_family(kind: serde_json::Value) -> Result<serde_json::Value, String> {
    conditions::condition_family(kind)
}

/// Port of `newGroupCondition` (`src/dae/groupConditions.ts:19`).
pub fn new_group_condition() -> Result<serde_json::Value, String> {
    conditions::new_group_condition()
}

/// Port of `conditionValues` (`src/dae/groupConditions.ts:21`).
pub fn condition_values(values: Vec<String>) -> Result<serde_json::Value, String> {
    conditions::condition_values(values)
}

/// Port of `parseConditionValues` (`src/dae/groupConditions.ts:22`).
pub fn parse_condition_values(text: String) -> Result<Option<Vec<String>>, String> {
    conditions::parse_condition_values(text)
}

/// Port of `parseGroupConditions` (`src/dae/groupConditions.ts:51`).
pub fn parse_group_conditions(source: String) -> Result<Option<Vec<serde_json::Value>>, String> {
    conditions::parse_group_conditions(source)
}

/// Port of `serializeGroupConditions` (`src/dae/groupConditions.ts:119`).
pub fn serialize_group_conditions(rows: serde_json::Value) -> Result<Option<String>, String> {
    conditions::serialize_group_conditions(rows)
}

/// Port of `groupFilterDraft` (`src/dae/groupConditions.ts:145`).
pub fn group_filter_draft(
    source: String,
    advanced: Option<bool>,
) -> Result<serde_json::Value, String> {
    conditions::group_filter_draft(source, advanced)
}

/// Port of `groupFilterText` (`src/dae/groupConditions.ts:149`).
pub fn group_filter_text(filter: serde_json::Value) -> Result<serde_json::Value, String> {
    conditions::group_filter_text(filter)
}

/// Port of `groupFilterTexts` (`src/dae/groupConditions.ts:152`).
pub fn group_filter_texts(filters: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    conditions::group_filter_texts(filters)
}

/// Port of `reconcileGroupFilters` (`src/dae/groupConditions.ts:154`).
pub fn reconcile_group_filters(
    previous: Vec<serde_json::Value>,
    texts: Vec<String>,
) -> Result<Vec<serde_json::Value>, String> {
    conditions::reconcile_group_filters(previous, texts)
}

/// Port of `newGroupFilter` (`src/dae/groupConditions.ts:161`).
pub fn new_group_filter() -> Result<serde_json::Value, String> {
    conditions::new_group_filter()
}

/// Port of `groupAdmits` (`src/dae/groupFilters.ts:59`).
pub fn group_admits(
    filters: Vec<String>,
    node: serde_json::Value,
) -> Result<serde_json::Value, String> {
    conditions::group_admits(filters, node)
}

/// Port of `readGroupEntries` (`src/dae/groups.ts:18`).
pub fn read_group_entries(text: String) -> Result<Vec<serde_json::Value>, String> {
    groups::read_group_entries(text)
}

/// Port of `quoteName` (`src/dae/groups.ts:43`).
pub fn quote_name(value: String) -> Result<serde_json::Value, String> {
    groups::quote_name(value)
}

/// Port of `isWritableName` (`src/dae/groups.ts:44`).
pub fn is_writable_name(value: String) -> Result<serde_json::Value, String> {
    groups::is_writable_name(value)
}

/// Port of `groupNameProblem` (`src/dae/groups.ts:47`).
pub fn group_name_problem(
    name: String,
    taken: serde_json::Value,
) -> Result<serde_json::Value, String> {
    groups::group_name_problem(name, taken)
}

/// Port of `writeGroupEntry` (`src/dae/groups.ts:60`).
pub fn write_group_entry(
    text: String,
    name: String,
    next: serde_json::Value,
) -> Result<String, String> {
    groups::write_group_entry(text, name, next)
}

/// Port of `nameText` (`src/dae/groups.ts:184`).
pub fn name_text(
    value: serde_json::Value,
    written: serde_json::Value,
) -> Result<serde_json::Value, String> {
    groups::name_text(value, written)
}

/// Port of `nestedIn` (`src/dae/groups.ts:190`).
pub fn nested_in(entry: serde_json::Value) -> Result<Vec<String>, String> {
    groups::nested_in(entry)
}

/// Port of `exactTokens` (`src/dae/groups.ts:228`).
pub fn exact_tokens(filter: String, call: serde_json::Value) -> Result<serde_json::Value, String> {
    groups::exact_tokens(filter, call)
}

/// Port of `namedIn` (`src/dae/groups.ts:242`).
pub fn named_in(entry: serde_json::Value) -> Result<Vec<String>, String> {
    groups::named_in(entry)
}

/// Port of `classifyFilters` (`src/dae/groups.ts:248`).
pub fn classify_filters(entry: serde_json::Value) -> Result<serde_json::Value, String> {
    groups::classify_filters(entry)
}

/// Port of `describeFilters` (`src/dae/groups.ts:259`).
pub fn describe_filters(filters: Vec<String>) -> Result<serde_json::Value, String> {
    groups::describe_filters(filters)
}

/// Port of `citingGroups` (`src/dae/groups.ts:291`).
pub fn citing_groups(text: String, tag: String) -> Result<serde_json::Value, String> {
    groups::citing_groups(text, tag)
}

/// Port of `namedInExpression` (`src/dae/groups.ts:295`).
pub fn named_in_expression(text: String, tag: String) -> Result<serde_json::Value, String> {
    groups::named_in_expression(text, tag)
}

/// Port of `groupsNamingTag` (`src/dae/groups.ts:298`).
pub fn groups_naming_tag(text: String, tag: String) -> Result<serde_json::Value, String> {
    groups::groups_naming_tag(text, tag)
}

/// Port of `addSubtagsToGroup` (`src/dae/groups.ts:335`).
pub fn add_subtags_to_group(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<serde_json::Value, String> {
    groups::add_subtags_to_group(text, group, tags)
}

/// Port of `addNodesToGroup` (`src/dae/groups.ts:336`).
pub fn add_nodes_to_group(
    text: String,
    group: String,
    names: Vec<String>,
) -> Result<serde_json::Value, String> {
    groups::add_nodes_to_group(text, group, names)
}

/// Port of `removeSubtagsFromGroup` (`src/dae/groups.ts:337`).
pub fn remove_subtags_from_group(
    text: String,
    group: String,
    tags: Vec<String>,
) -> Result<serde_json::Value, String> {
    groups::remove_subtags_from_group(text, group, tags)
}

/// Port of `splitTop` (`src/dae/groups.ts:339`).
pub fn split_top(text: String, separator: String) -> Result<Vec<String>, String> {
    groups::split_top(text, separator)
}

/// Port of `ruleCondition` (`src/dae/groups.ts:387`).
pub fn rule_condition(kind: serde_json::Value, value: String) -> Result<Option<String>, String> {
    groups::rule_condition(kind, value)
}

/// Port of `includePatterns` (`src/dae/newSource.ts:5`).
pub fn include_patterns(text: String) -> Result<Vec<String>, String> {
    new_source::include_patterns(text)
}

/// Port of `isGlob` (`src/dae/newSource.ts:13`).
pub fn is_glob(pattern: String) -> Result<serde_json::Value, String> {
    new_source::is_glob(pattern)
}

/// Port of `globMatch` (`src/dae/newSource.ts:16`).
pub fn glob_match(pattern: String, path: String) -> Result<bool, String> {
    new_source::glob_match(pattern, path)
}

/// Port of `includedBy` (`src/dae/newSource.ts:25`).
pub fn included_by(patterns: Vec<String>, path: String) -> Result<serde_json::Value, String> {
    new_source::included_by(patterns, path)
}

/// Port of `resolveIncludePath` (`src/dae/newSource.ts:29`).
pub fn resolve_include_path(
    base: Option<String>,
    path: String,
) -> Result<serde_json::Value, String> {
    new_source::resolve_include_path(base, path)
}

/// Port of `includeDirectory` (`src/dae/newSource.ts:51`).
pub fn include_directory(patterns: Vec<String>) -> Result<String, String> {
    new_source::include_directory(patterns)
}

/// Port of `newSourcePathProblem` (`src/dae/newSource.ts:60`).
pub fn new_source_path_problem(path: String) -> Result<serde_json::Value, String> {
    new_source::new_source_path_problem(path)
}

/// Port of `namePatterns` (`src/dae/newSource.ts:74`).
pub fn name_patterns(patterns: Vec<String>) -> Result<Vec<serde_json::Value>, String> {
    new_source::name_patterns(patterns)
}

/// Port of `newSourceNameProblem` (`src/dae/newSource.ts:90`).
pub fn new_source_name_problem(
    name: String,
    pattern: serde_json::Value,
) -> Result<serde_json::Value, String> {
    new_source::new_source_name_problem(name, pattern)
}

/// Port of `sourceAt` (`src/dae/newSource.ts:98`).
pub fn source_at(
    sources: Vec<serde_json::Value>,
    path: String,
) -> Result<Option<serde_json::Value>, String> {
    new_source::source_at(sources, path)
}

/// Port of `isProtocol` (`src/dae/nodeForm.ts:3`).
pub fn is_protocol(value: String) -> Result<bool, String> {
    node_form::is_protocol(value)
}

/// Port of `protocolName` (`src/dae/nodeForm.ts:5`).
pub fn protocol_name(protocol: serde_json::Value) -> Result<serde_json::Value, String> {
    node_form::protocol_name(protocol)
}

/// Port of `emptyNode` (`src/dae/nodeForm.ts:11`).
pub fn empty_node(protocol: serde_json::Value) -> Result<serde_json::Value, String> {
    node_form::empty_node(protocol)
}

/// Port of `parseNodeLink` (`src/dae/nodeForm.ts:52`).
pub fn parse_node_link(original: String) -> Result<serde_json::Value, String> {
    node_form::parse_node_link(original)
}

/// Port of `nodeLink` (`src/dae/nodeForm.ts:188`).
pub fn node_link(
    fields: serde_json::Value,
    document: Option<serde_json::Value>,
) -> Result<String, String> {
    node_form::node_link(fields, document)
}

/// Port of `securityOptions` (`src/dae/nodeForm.ts:264`).
pub fn security_options(protocol: serde_json::Value) -> Result<serde_json::Value, String> {
    node_form::security_options(protocol)
}

/// Port of `usesTls` (`src/dae/nodeForm.ts:267`).
pub fn uses_tls(f: serde_json::Value) -> Result<serde_json::Value, String> {
    node_form::uses_tls(f)
}

/// Port of `manualFields` (`src/dae/nodeForm.ts:273`).
pub fn manual_fields(f: serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    node_form::manual_fields(f)
}

/// Port of `advancedFields` (`src/dae/nodeForm.ts:312`).
pub fn advanced_fields(f: serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    node_form::advanced_fields(f)
}

/// Port of `settle` (`src/dae/nodeForm.ts:356`).
pub fn settle(f: serde_json::Value) -> Result<serde_json::Value, String> {
    node_form::settle(f)
}

/// Port of `manualInvalid` (`src/dae/nodeForm.ts:392`).
pub fn manual_invalid(f: serde_json::Value) -> Result<Vec<String>, String> {
    node_form::manual_invalid(f)
}

/// Port of `manualLink` (`src/dae/nodeForm.ts:408`).
pub fn manual_link(f: serde_json::Value) -> Result<String, String> {
    node_form::manual_link(f)
}

/// Port of `isNodeLink` (`src/dae/nodes.ts:16`).
pub fn is_node_link(value: String) -> Result<serde_json::Value, String> {
    nodes::is_node_link(value)
}

/// Port of `readNodeEntries` (`src/dae/nodes.ts:19`).
pub fn read_node_entries(text: String) -> Result<Vec<serde_json::Value>, String> {
    nodes::read_node_entries(text)
}

/// Port of `readChainEntries` (`src/dae/nodes.ts:39`).
pub fn read_chain_entries(text: String) -> Result<Vec<serde_json::Value>, String> {
    nodes::read_chain_entries(text)
}

/// Port of `nodeReferenced` (`src/dae/nodes.ts:115`).
pub fn node_referenced(text: String, name: String) -> Result<serde_json::Value, String> {
    nodes::node_referenced(text, name)
}

/// Port of `nodeReferrers` (`src/dae/nodes.ts:116`).
pub fn node_referrers(text: String, name: String) -> Result<Vec<serde_json::Value>, String> {
    nodes::node_referrers(text, name)
}

/// Port of `removeChainEntry` (`src/dae/nodes.ts:130`).
pub fn remove_chain_entry(text: String, original: serde_json::Value) -> Result<String, String> {
    nodes::remove_chain_entry(text, original)
}

/// Port of `chainHopCandidate` (`src/dae/nodes.ts:143`).
pub fn chain_hop_candidate(node: serde_json::Value) -> Result<serde_json::Value, String> {
    nodes::chain_hop_candidate(node)
}

/// Port of `tunnelDialable` (`src/dae/nodes.ts:145`).
pub fn tunnel_dialable(node: serde_json::Value) -> Result<serde_json::Value, String> {
    nodes::tunnel_dialable(node)
}

/// Port of `chainHopProblem` (`src/dae/nodes.ts:147`).
pub fn chain_hop_problem(
    nodes_list: Vec<serde_json::Value>,
    name: String,
    index: f64,
) -> Result<Option<String>, String> {
    nodes::chain_hop_problem(nodes_list, name, index)
}

/// Port of `chainUdpProblem` (`src/dae/nodes.ts:154`).
pub fn chain_udp_problem(hops: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    nodes::chain_udp_problem(hops)
}

/// Port of `writeChainEntry` (`src/dae/nodes.ts:171`).
pub fn write_chain_entry(
    text: String,
    original: serde_json::Value,
    next: serde_json::Value,
) -> Result<String, String> {
    nodes::write_chain_entry(text, original, next)
}

/// Port of `appendChainEntry` (`src/dae/nodes.ts:180`).
pub fn append_chain_entry(text: String, next: serde_json::Value) -> Result<String, String> {
    nodes::append_chain_entry(text, next)
}

/// Port of `writeNodeEntry` (`src/dae/nodes.ts:196`).
pub fn write_node_entry(
    text: String,
    original: serde_json::Value,
    next: serde_json::Value,
) -> Result<String, String> {
    nodes::write_node_entry(text, original, next)
}

/// Port of `readRuleFallback` (`src/dae/outboundMode.ts:32`).
pub fn read_rule_fallback(text: String) -> Result<String, String> {
    outbound_mode::read_rule_fallback(text)
}

/// Port of `writeRuleFallback` (`src/dae/outboundMode.ts:44`).
pub fn write_rule_fallback(text: String, target: String) -> Result<String, String> {
    outbound_mode::write_rule_fallback(text, target)
}

/// Port of `readMode` (`src/dae/outboundMode.ts:78`).
pub fn read_mode(text: String) -> Result<serde_json::Value, String> {
    outbound_mode::read_mode(text)
}

/// Port of `writeMode` (`src/dae/outboundMode.ts:84`).
pub fn write_mode(text: String, next: serde_json::Value) -> Result<String, String> {
    outbound_mode::write_mode(text, next)
}

/// Port of `sameMode` (`src/dae/outboundMode.ts:131`).
pub fn same_mode(a: serde_json::Value, b: serde_json::Value) -> Result<serde_json::Value, String> {
    outbound_mode::same_mode(a, b)
}

/// Port of `modeRuleLines` (`src/dae/outboundMode.ts:133`).
pub fn mode_rule_lines(text: String) -> Result<serde_json::Value, String> {
    outbound_mode::mode_rule_lines(text)
}

/// Port of `nodeProbeSupport` (`src/dae/probes.ts:21`).
pub fn node_probe_support(protocol: Option<String>) -> Result<serde_json::Value, String> {
    geodata::node_probe_support(protocol)
}

/// Port of `supportsNodeProbe` (`src/dae/probes.ts:26`).
pub fn supports_node_probe(
    protocol: Option<String>,
    kind: serde_json::Value,
    transports: serde_json::Value,
) -> Result<bool, String> {
    geodata::supports_node_probe(protocol, kind, transports)
}

/// Port of `regionFlag` (`src/dae/regions.ts:95`).
pub fn region_flag(region: String) -> Result<serde_json::Value, String> {
    regions::region_flag(region)
}

/// Port of `matchRegion` (`src/dae/regions.ts:159`).
pub fn match_region(name: String) -> Result<serde_json::Value, String> {
    regions::match_region(name)
}

/// Port of `sourceFor` (`src/dae/ruleText.ts:4`).
pub fn source_for(
    list: serde_json::Value,
    source: serde_json::Value,
) -> Result<serde_json::Value, String> {
    rule_text::source_for(list, source)
}

/// Port of `ruleAnchor` (`src/dae/ruleText.ts:83`).
pub fn rule_anchor(
    source: serde_json::Value,
    rule: serde_json::Value,
    scan: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>, String> {
    rule_text::rule_anchor(source, rule, scan)
}

/// Port of `dnsRuleTarget` (`src/dae/ruleText.ts:99`).
pub fn dns_rule_target(rule: serde_json::Value) -> Result<serde_json::Value, String> {
    rule_text::dns_rule_target(rule)
}

/// Port of `dnsRuleAnchor` (`src/dae/ruleText.ts:107`).
pub fn dns_rule_anchor(
    source: serde_json::Value,
    rule: serde_json::Value,
    list: String,
    scan: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>, String> {
    rule_text::dns_rule_anchor(source, rule, list, scan)
}

/// Port of `dnsListEnd` (`src/dae/ruleText.ts:124`).
pub fn dns_list_end(
    sources: Vec<serde_json::Value>,
    list: String,
) -> Result<Option<serde_json::Value>, String> {
    rule_text::dns_list_end(sources, list)
}

/// Port of `dnsUpstreams` (`src/dae/ruleText.ts:153`).
pub fn dns_upstreams(
    text: String,
    scan: Option<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, String> {
    rule_text::dns_upstreams(text, scan)
}

/// Port of `dnsUpstreamNames` (`src/dae/ruleText.ts:164`).
pub fn dns_upstream_names(
    text: String,
    scan: Option<serde_json::Value>,
) -> Result<Vec<String>, String> {
    rule_text::dns_upstream_names(text, scan)
}

/// Port of `ruleLine` (`src/dae/ruleText.ts:166`).
pub fn rule_line(
    condition: String,
    outbound: String,
    must: Option<bool>,
) -> Result<serde_json::Value, String> {
    rule_text::rule_line(condition, outbound, must)
}

/// Port of `addRule` (`src/dae/ruleText.ts:168`).
pub fn add_rule(
    text: String,
    anchor: serde_json::Value,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    rule_text::add_rule(text, anchor, condition, outbound, must)
}

/// Port of `replaceRuleTarget` (`src/dae/ruleText.ts:179`).
pub fn replace_rule_target(
    text: String,
    anchor: serde_json::Value,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    rule_text::replace_rule_target(text, anchor, outbound, must)
}

/// Port of `removeRule` (`src/dae/ruleText.ts:187`).
pub fn remove_rule(text: String, anchor: serde_json::Value) -> Result<Option<String>, String> {
    rule_text::remove_rule(text, anchor)
}

/// Port of `replaceRule` (`src/dae/ruleText.ts:192`).
pub fn replace_rule(
    text: String,
    anchor: serde_json::Value,
    condition: String,
    outbound: String,
    must: bool,
) -> Result<Option<String>, String> {
    rule_text::replace_rule(text, anchor, condition, outbound, must)
}

/// Port of `addFallback` (`src/dae/ruleText.ts:205`).
pub fn add_fallback(
    text: String,
    anchor: serde_json::Value,
    target: String,
) -> Result<Option<String>, String> {
    rule_text::add_fallback(text, anchor, target)
}

/// Port of `numberSetting` (`src/dae/settings.ts:28`).
pub fn number_setting(field: serde_json::Value) -> Result<bool, String> {
    settings::number_setting(field)
}

/// Port of `settingValue` (`src/dae/settings.ts:38`).
pub fn setting_value(field: serde_json::Value, value: String) -> Result<String, String> {
    settings::setting_value(field, value)
}

/// Port of `serializeSetting` (`src/dae/settings.ts:58`).
pub fn serialize_setting(
    field: serde_json::Value,
    value: String,
) -> Result<Option<String>, String> {
    settings::serialize_setting(field, value)
}

/// Port of `writeSettings` (`src/dae/settings.ts:88`).
pub fn write_settings(
    text: String,
    section: serde_json::Value,
    index: f64,
    patch: serde_json::Value,
) -> Result<String, String> {
    settings::write_settings(text, section, index, patch)
}

/// Port of `holdsDns` (`src/dae/setup.ts:33`).
pub fn holds_dns(texts: Vec<String>) -> Result<bool, String> {
    templates::holds_dns(texts)
}

/// Port of `holdsRouting` (`src/dae/setup.ts:37`).
pub fn holds_routing(texts: Vec<String>) -> Result<bool, String> {
    templates::holds_routing(texts)
}

/// Port of `templateGroups` (`src/dae/setup.ts:51`).
pub fn template_groups(rules: String, defined: Vec<String>) -> Result<Vec<String>, String> {
    templates::template_groups(rules, defined)
}

/// Port of `allGroupNames` (`src/dae/sources.ts:18`).
pub fn all_group_names(
    sources: Vec<serde_json::Value>,
    draft: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    templates::all_group_names(sources, draft)
}

/// Port of `allUpstreamNames` (`src/dae/sources.ts:20`).
pub fn all_upstream_names(
    sources: Vec<serde_json::Value>,
    draft: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    templates::all_upstream_names(sources, draft)
}

/// Port of `redacted` (`src/dae/sources.ts:24`).
pub fn redacted(source: serde_json::Value) -> Result<serde_json::Value, String> {
    templates::redacted(source)
}

/// Port of `fileName` (`src/dae/sources.ts:25`).
pub fn file_name(source: serde_json::Value) -> Result<serde_json::Value, String> {
    templates::file_name(source)
}

/// Port of `validationSources` (`src/dae/sources.ts:29`).
pub fn validation_sources(
    sources: Vec<serde_json::Value>,
    replacement: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>, String> {
    templates::validation_sources(sources, replacement)
}

/// Port of `restartSettings` (`src/dae/sources.ts:45`).
pub fn restart_settings(
    diagnostics: Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, String> {
    templates::restart_settings(diagnostics)
}

/// Port of `restartRequired` (`src/dae/sources.ts:49`).
pub fn restart_required(diagnostics: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    templates::restart_required(diagnostics)
}

/// Port of `unwrittenRestartSettings` (`src/dae/sources.ts:55`).
pub fn unwritten_restart_settings(
    settings_list: Vec<serde_json::Value>,
    sources: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    templates::unwritten_restart_settings(settings_list, sources)
}

/// Port of `completeSubscriptionUrl` (`src/dae/subscriptions.ts:16`).
pub fn complete_subscription_url(value: String) -> Result<serde_json::Value, String> {
    subscriptions::complete_subscription_url(value)
}

/// Port of `isSubscriptionUrl` (`src/dae/subscriptions.ts:26`).
pub fn is_subscription_url(value: String) -> Result<serde_json::Value, String> {
    subscriptions::is_subscription_url(value)
}

/// Port of `parseInterval` (`src/dae/subscriptions.ts:61`).
pub fn parse_interval(value: String) -> Result<serde_json::Value, String> {
    subscriptions::parse_interval(value)
}

/// Port of `urlHost` (`src/dae/subscriptions.ts:73`).
pub fn url_host(url: Option<String>) -> Result<serde_json::Value, String> {
    subscriptions::url_host(url)
}

/// Port of `agentProblem` (`src/dae/subscriptions.ts:84`).
pub fn agent_problem(agent: String, written: bool) -> Result<Option<serde_json::Value>, String> {
    subscriptions::agent_problem(agent, written)
}

/// Port of `readSubscriptionEntries` (`src/dae/subscriptions.ts:294`).
pub fn read_subscription_entries(text: String) -> Result<Vec<serde_json::Value>, String> {
    subscriptions::read_subscription_entries(text)
}

/// Port of `writeSubscriptionEntry` (`src/dae/subscriptions.ts:328`).
pub fn write_subscription_entry(
    text: String,
    tag: String,
    change: serde_json::Value,
) -> Result<String, String> {
    subscriptions::write_subscription_entry(text, tag, change)
}

/// Port of `templateRules` (`src/dae/templates.ts:213`).
pub fn template_rules(
    template: String,
    options: Option<serde_json::Value>,
) -> Result<Vec<String>, String> {
    templates::template_rules(template, options)
}

/// Port of `detectTemplate` (`src/dae/templates.ts:258`).
pub fn detect_template(text: String) -> Result<Option<serde_json::Value>, String> {
    templates::detect_template(text)
}

/// Port of `routingOptions` (`src/dae/templates.ts:293`).
pub fn routing_options(text: String) -> Result<Option<serde_json::Value>, String> {
    templates::routing_options(text)
}

/// Port of `isPresetRouting` (`src/dae/templates.ts:320`).
pub fn is_preset_routing(text: String) -> Result<bool, String> {
    templates::is_preset_routing(text)
}

/// Port of `isBareName` (`src/dae/text.ts:4`).
pub fn is_bare_name(value: String) -> Result<serde_json::Value, String> {
    text::is_bare_name(value)
}

/// Port of `quote` (`src/dae/text.ts:5`).
pub fn quote(value: String) -> Result<String, String> {
    text::quote(value)
}

/// Port of `isQuotable` (`src/dae/text.ts:10`).
pub fn is_quotable(value: String) -> Result<bool, String> {
    text::is_quotable(value)
}

/// Port of `scanConfig` (`src/dae/text.ts:33`).
pub fn scan_config(text: String) -> Result<serde_json::Value, String> {
    text::scan_config(text)
}

/// Port of `isFragment` (`src/dae/text.ts:109`).
pub fn is_fragment(text: String) -> Result<bool, String> {
    text::is_fragment(text)
}

/// Port of `unquote` (`src/dae/text.ts:123`).
pub fn unquote(text: String) -> Result<serde_json::Value, String> {
    text::unquote(text)
}

/// Port of `uncomment` (`src/dae/text.ts:124`).
pub fn uncomment(text: String) -> Result<String, String> {
    text::uncomment(text)
}

/// Port of `blockFields` (`src/dae/text.ts:130`).
pub fn block_fields(
    text: String,
    block: serde_json::Value,
    tokens: Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, String> {
    text::block_fields(text, block, tokens)
}

/// Port of `blockBody` (`src/dae/text.ts:155`).
pub fn block_body(text: String, block: serde_json::Value) -> Result<Vec<String>, String> {
    text::block_body(text, block)
}

/// Port of `blockEntries` (`src/dae/text.ts:165`).
pub fn block_entries(
    text: String,
    block: serde_json::Value,
) -> Result<Vec<serde_json::Value>, String> {
    text::block_entries(text, block)
}

/// Port of `policyKind` (`src/dae/vocab.ts:63`).
pub fn policy_kind(native: String) -> Result<Option<serde_json::Value>, String> {
    vocab::policy_kind(native)
}

/// Port of `isBuiltinOutbound` (`src/dae/vocab.ts:87`).
pub fn is_builtin_outbound(name: Option<String>) -> Result<bool, String> {
    vocab::is_builtin_outbound(name)
}

/// Port of `readXhttp` (`src/dae/xhttp.ts:101`).
pub fn read_xhttp(link: String) -> Result<Option<serde_json::Value>, String> {
    xhttp::read_xhttp(link)
}

/// Port of `duplicateXhttpHeaders` (`src/dae/xhttp.ts:106`).
pub fn duplicate_xhttp_headers(headers: serde_json::Value) -> Result<serde_json::Value, String> {
    xhttp::duplicate_xhttp_headers(headers)
}

/// Port of `writeXhttp` (`src/dae/xhttp.ts:118`).
pub fn write_xhttp(link: String, change: serde_json::Value) -> Result<String, String> {
    xhttp::write_xhttp(link, change)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn text_scanner_utf16_and_quoting() {
        let cfg = "group {\n  '香港👍' {\n    policy: score # ok\n  }\n}\n";
        let scanned = scan_config(cfg.to_string()).expect("scan");
        let blocks = scanned["blocks"].as_array().expect("blocks");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["name"], "group");
        let child = &blocks[0]["children"][0];
        assert_eq!(child["name"], "香港👍");
        // Verify UTF-16 offsets account for the 2-unit emoji.
        assert_eq!(child["open"].as_u64(), Some(17));
        assert_eq!(quote("abc".to_string()).as_deref(), Ok("'abc'"));
        assert!(quote("a'b".to_string()).is_err());
        assert!(quote("a\\".to_string()).is_err());
        assert_eq!(quote("a\\\\".to_string()).as_deref(), Ok("'a\\\\'"));
        assert!(is_fragment("domain(geosite: cn)".to_string()).unwrap());
        assert!(!is_fragment("domain(a) # x".to_string()).unwrap());
    }

    #[test]
    fn regions_and_flags_roundtrip() {
        assert_eq!(region_flag("HK".to_string()).unwrap(), json!("🇭🇰"));
        assert_eq!(
            match_region("Hong Kong 01".to_string()).unwrap(),
            json!("HK")
        );
        assert_eq!(match_region("🇯🇵 Tokyo".to_string()).unwrap(), json!("JP"));
        assert_eq!(
            match_region("plain node".to_string()).unwrap(),
            serde_json::Value::Null
        );
        assert_eq!(has_embedded_flag("🇭🇰 HK".to_string()).unwrap(), json!(true));
        assert!(valid_flag_choice(json!("US")).unwrap());
        assert!(valid_flag_choice(json!("none")).unwrap());
        assert!(!valid_flag_choice(json!("ZZ")).unwrap());
        assert!(flag_choices("en".to_string()).is_ok());
        assert!(flag_choices("".to_string()).is_err());
    }

    #[test]
    fn groups_and_conditions_work() {
        let cond =
            rule_condition(json!("domainSuffix"), "example.com, foo.org".to_string()).unwrap();
        assert_eq!(
            cond.as_deref(),
            Some("domain(suffix: example.com, suffix: foo.org)")
        );
        let parsed = parse_conditions(
            "!domain(geosite: cn) && dport(443)".to_string(),
            vec!["geosite".into(), "dport".into()],
        )
        .unwrap()
        .unwrap();
        assert_eq!(parsed.len(), 2);
        let serialized = serialize_conditions(serde_json::Value::Array(parsed)).unwrap();
        assert_eq!(
            serialized.as_deref(),
            Some("!domain(geosite: cn) && dport(443)")
        );

        assert_eq!(
            group_admits(
                vec!["subtag(airport) && !name(regex: '^HK')".into()],
                json!({"name": "JP 01", "subscription_tag": "airport"})
            )
            .unwrap(),
            json!(true)
        );
        assert_eq!(
            group_admits(
                vec!["subtag(airport) && !name(regex: '^HK')".into()],
                json!({"name": "HK 01", "subscription_tag": "airport"})
            )
            .unwrap(),
            json!(false)
        );
    }

    #[test]
    fn nodes_subscriptions_and_xhttp_work() {
        let link = "vless://11111111-1111-1111-1111-111111111111@example.com:443?type=xhttp&security=tls#demo";
        let parsed = parse_node_link(link.to_string()).unwrap();
        assert_eq!(parsed["fields"]["protocol"], "vless");
        assert_eq!(parsed["fields"]["host"], "example.com");
        let rebuilt =
            node_link(parsed["fields"].clone(), Some(parsed["document"].clone())).unwrap();
        assert_eq!(rebuilt, link);

        let x = read_xhttp(link.to_string()).unwrap().unwrap();
        assert_eq!(x["mode"], "");
        let updated = write_xhttp(link.to_string(), json!({"mode": "stream-up"})).unwrap();
        assert!(updated.contains("mode=stream-up"));

        assert_eq!(parse_interval("1500ms".to_string()).unwrap(), json!(2));
        assert_eq!(parse_interval("2h".to_string()).unwrap(), json!(7200));
        assert_eq!(
            complete_subscription_url("example.com/sub".to_string()).unwrap(),
            json!("https://example.com/sub")
        );
    }

    #[test]
    fn outbound_mode_and_templates_work() {
        let cfg = "routing {\n  dip(geoip: private) -> direct(must)\n  fallback: proxy\n}\n";
        assert_eq!(read_rule_fallback(cfg.to_string()).unwrap(), "proxy");
        let global_cfg = write_mode(
            cfg.to_string(),
            json!({"mode": "global", "target": "proxy"}),
        )
        .unwrap();
        assert_eq!(
            read_mode(global_cfg.clone()).unwrap(),
            json!({"mode": "global", "target": "proxy"})
        );
        let back = write_mode(global_cfg, json!({"mode": "rule"})).unwrap();
        assert_eq!(back, cfg);

        let rules = template_rules("global".to_string(), None).unwrap();
        assert!(!rules.is_empty());
        assert_eq!(
            policy_kind("MIN_MOVING_AVG".to_string()).unwrap(),
            Some(json!("urltest"))
        );
    }

    #[test]
    fn settings_and_new_source_work() {
        assert!(glob_match("conf/*.dae".to_string(), "conf/a.dae".to_string()).unwrap());
        assert!(!glob_match("conf/*.dae".to_string(), "conf/sub/a.dae".to_string()).unwrap());
        assert_eq!(
            resolve_include_path(
                Some("/etc/dae/config.dae".into()),
                "./rules/../extra.dae".into()
            )
            .unwrap(),
            json!("/etc/dae/extra.dae")
        );
        let field = json!({"key": "tproxy_port", "type": "integer", "max": "65535"});
        assert!(number_setting(field.clone()).unwrap());
        assert_eq!(
            serialize_setting(field, "12345".into()).unwrap().as_deref(),
            Some("12345")
        );
    }
}
