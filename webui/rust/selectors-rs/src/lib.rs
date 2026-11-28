//! JS bindings for `src/api/selectors.ts`: generated from the signatures in `port.rs` by rust-anything.
//! Regenerate with `rust-anything refresh` after changing them.

use wasm_bindgen::prelude::*;

pub mod js;
pub mod port;
pub mod wire;

#[allow(unused_imports)]
use serde_json::Value;
pub mod utf16;

/// `safeHttpUrl`.
fn safe_http_url_inner(value: String) -> Result<bool, String> {
    let out = port::safe_http_url(value)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "safeHttpUrl")]
pub fn safe_http_url(value: String) -> Result<bool, JsError> {
    safe_http_url_inner(value).map_err(|e| JsError::new(&e))
}

/// `measured`.
fn measured_inner(h: String) -> Result<String, String> {
    let h = crate::js::from_json(&h)?;
    let out = port::measured(h)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "measured")]
pub fn measured(h: String) -> Result<String, JsError> {
    measured_inner(h).map_err(|e| JsError::new(&e))
}

/// `preferredObservation`.
fn preferred_observation_inner(health: String) -> Result<String, String> {
    let health = crate::js::from_json(&health)?;
    let out = port::preferred_observation(health)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "preferredObservation")]
pub fn preferred_observation(health: String) -> Result<String, JsError> {
    preferred_observation_inner(health).map_err(|e| JsError::new(&e))
}

/// `foldFamilies`.
fn fold_families_inner(rows: String) -> Result<String, String> {
    let rows = crate::js::from_json(&rows)?;
    let out = port::fold_families(rows)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "foldFamilies")]
pub fn fold_families(rows: String) -> Result<String, JsError> {
    fold_families_inner(rows).map_err(|e| JsError::new(&e))
}

/// `preferredHealth`.
fn preferred_health_inner(node: String) -> Result<String, String> {
    let node = crate::js::from_json(&node)?;
    let out = port::preferred_health(node)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "preferredHealth")]
pub fn preferred_health(node: String) -> Result<String, JsError> {
    preferred_health_inner(node).map_err(|e| JsError::new(&e))
}

/// `probeKinds`.
fn probe_kinds_inner(health: String) -> Result<String, String> {
    let health = crate::js::from_json(&health)?;
    let out = port::probe_kinds(health)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "probeKinds")]
pub fn probe_kinds(health: String) -> Result<String, JsError> {
    probe_kinds_inner(health).map_err(|e| JsError::new(&e))
}

/// `healthMillis`.
fn health_millis_inner(health: String) -> Result<String, String> {
    let health = crate::js::from_json(&health)?;
    let out = port::health_millis(health)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "healthMillis")]
pub fn health_millis(health: String) -> Result<String, JsError> {
    health_millis_inner(health).map_err(|e| JsError::new(&e))
}

/// `compareLatency`.
fn compare_latency_inner(a: Option<f64>, b: Option<f64>) -> Result<String, String> {
    let out = port::compare_latency(a, b)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "compareLatency")]
pub fn compare_latency(a: Option<f64>, b: Option<f64>) -> Result<String, JsError> {
    compare_latency_inner(a, b).map_err(|e| JsError::new(&e))
}

/// `outboundUsage`.
fn outbound_usage_inner(snapshot: String) -> Result<String, String> {
    let snapshot = crate::js::from_json(&snapshot)?;
    let out = port::outbound_usage(snapshot)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "outboundUsage")]
pub fn outbound_usage(snapshot: String) -> Result<String, JsError> {
    outbound_usage_inner(snapshot).map_err(|e| JsError::new(&e))
}

/// `simulatedAddress`.
fn simulated_address_inner(result: String) -> Result<String, String> {
    let result = crate::js::from_json(&result)?;
    let out = port::simulated_address(result)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "simulatedAddress")]
pub fn simulated_address(result: String) -> Result<String, JsError> {
    simulated_address_inner(result).map_err(|e| JsError::new(&e))
}

/// `ipLiteral`.
fn ip_literal_inner(text: String) -> Result<Option<String>, String> {
    let out = port::ip_literal(text)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "ipLiteral")]
pub fn ip_literal(text: String) -> Result<Option<String>, JsError> {
    ip_literal_inner(text).map_err(|e| JsError::new(&e))
}

/// `pseudoOwner`.
fn pseudo_owner_inner(node: String) -> Result<String, String> {
    let node = crate::js::from_json(&node)?;
    let out = port::pseudo_owner(node)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "pseudoOwner")]
pub fn pseudo_owner(node: String) -> Result<String, JsError> {
    pseudo_owner_inner(node).map_err(|e| JsError::new(&e))
}

/// `noNodeSources`.
fn no_node_sources_inner(providers: String, nodes: String) -> Result<bool, String> {
    let providers = crate::js::from_json(&providers)?;
    let nodes = crate::js::from_json(&nodes)?;
    let out = port::no_node_sources(providers, nodes)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "noNodeSources")]
pub fn no_node_sources(providers: String, nodes: String) -> Result<bool, JsError> {
    no_node_sources_inner(providers, nodes).map_err(|e| JsError::new(&e))
}

/// `pseudoOwnerId`.
fn pseudo_owner_id_inner(kind: String, providers: String) -> Result<String, String> {
    let providers = crate::js::from_json(&providers)?;
    let out = port::pseudo_owner_id(kind, providers)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "pseudoOwnerId")]
pub fn pseudo_owner_id(kind: String, providers: String) -> Result<String, JsError> {
    pseudo_owner_id_inner(kind, providers).map_err(|e| JsError::new(&e))
}

/// `nodeOwner`.
fn node_owner_inner(node: String, providers: String) -> Result<String, String> {
    let node = crate::js::from_json(&node)?;
    let providers = crate::js::from_json(&providers)?;
    let out = port::node_owner(node, providers)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "nodeOwner")]
pub fn node_owner(node: String, providers: String) -> Result<String, JsError> {
    node_owner_inner(node, providers).map_err(|e| JsError::new(&e))
}

/// `sourceIp`.
fn source_ip_inner(src: Option<String>) -> Result<Option<String>, String> {
    let out = port::source_ip(src)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "sourceIp")]
pub fn source_ip(src: Option<String>) -> Result<Option<String>, JsError> {
    source_ip_inner(src).map_err(|e| JsError::new(&e))
}

/// `addressPort`.
fn address_port_inner(address: Option<String>) -> Result<String, String> {
    let out = port::address_port(address)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "addressPort")]
pub fn address_port(address: Option<String>) -> Result<String, JsError> {
    address_port_inner(address).map_err(|e| JsError::new(&e))
}

/// `chainNames`.
fn chain_names_inner(chain: Vec<String>, names: String) -> Result<String, String> {
    let names = crate::js::from_json(&names)?;
    let out = port::chain_names(chain, names)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "chainNames")]
pub fn chain_names(chain: Vec<String>, names: String) -> Result<String, JsError> {
    chain_names_inner(chain, names).map_err(|e| JsError::new(&e))
}

/// `nodeSubscription`.
fn node_subscription_inner(row: String, subscriptions: String) -> Result<String, String> {
    let row = crate::js::from_json(&row)?;
    let subscriptions = crate::js::from_json(&subscriptions)?;
    let out = port::node_subscription(row, subscriptions)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "nodeSubscription")]
pub fn node_subscription(row: String, subscriptions: String) -> Result<String, JsError> {
    node_subscription_inner(row, subscriptions).map_err(|e| JsError::new(&e))
}

/// `lifecycleTone`.
fn lifecycle_tone_inner(state: String) -> Result<String, String> {
    let state = crate::js::from_json(&state)?;
    let out = port::lifecycle_tone(state)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "lifecycleTone")]
pub fn lifecycle_tone(state: String) -> Result<String, JsError> {
    lifecycle_tone_inner(state).map_err(|e| JsError::new(&e))
}

/// `memoryTone`.
fn memory_tone_inner(percent: f64) -> Result<String, String> {
    let out = port::memory_tone(percent)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "memoryTone")]
pub fn memory_tone(percent: f64) -> Result<String, JsError> {
    memory_tone_inner(percent).map_err(|e| JsError::new(&e))
}

/// `closedAllTone`.
fn closed_all_tone_inner(tally: String) -> Result<String, String> {
    let tally = crate::js::from_json(&tally)?;
    let out = port::closed_all_tone(tally)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "closedAllTone")]
pub fn closed_all_tone(tally: String) -> Result<String, JsError> {
    closed_all_tone_inner(tally).map_err(|e| JsError::new(&e))
}

/// `connectionRows`.
fn connection_rows_inner(snapshot: String) -> Result<String, String> {
    let snapshot = crate::js::from_json(&snapshot)?;
    let out = port::connection_rows(snapshot)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "connectionRows")]
pub fn connection_rows(snapshot: String) -> Result<String, JsError> {
    connection_rows_inner(snapshot).map_err(|e| JsError::new(&e))
}

/// `routineGap`.
fn routine_gap_inner(event: String) -> Result<bool, String> {
    let event = crate::js::from_json(&event)?;
    let out = port::routine_gap(event)?;
    Ok(out)
}

#[wasm_bindgen(js_name = "routineGap")]
pub fn routine_gap(event: String) -> Result<bool, JsError> {
    routine_gap_inner(event).map_err(|e| JsError::new(&e))
}

/// `shortId`.
fn short_id_inner(id: String) -> Result<String, String> {
    let out = port::short_id(id)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "shortId")]
pub fn short_id(id: String) -> Result<String, JsError> {
    short_id_inner(id).map_err(|e| JsError::new(&e))
}

/// `resolveSelectedLeaf`.
fn resolve_selected_leaf_inner(
    outbound: String,
    network: String,
    groups_by_name: String,
    groups_by_id: String,
    nodes_by_id: String,
) -> Result<String, String> {
    let groups_by_name = crate::js::from_json(&groups_by_name)?;
    let groups_by_id = crate::js::from_json(&groups_by_id)?;
    let nodes_by_id = crate::js::from_json(&nodes_by_id)?;
    let out =
        port::resolve_selected_leaf(outbound, network, groups_by_name, groups_by_id, nodes_by_id)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "resolveSelectedLeaf")]
pub fn resolve_selected_leaf(
    outbound: String,
    network: String,
    groups_by_name: String,
    groups_by_id: String,
    nodes_by_id: String,
) -> Result<String, JsError> {
    resolve_selected_leaf_inner(outbound, network, groups_by_name, groups_by_id, nodes_by_id)
        .map_err(|e| JsError::new(&e))
}

/// `interestingNotice`.
fn interesting_notice_inner(event: String) -> Result<String, String> {
    let event = crate::js::from_json(&event)?;
    let out = port::interesting_notice(event)?;
    crate::js::to_json(&out)
}

#[wasm_bindgen(js_name = "interestingNotice")]
pub fn interesting_notice(event: String) -> Result<String, JsError> {
    interesting_notice_inner(event).map_err(|e| JsError::new(&e))
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

static S_MEASURED_0: wire::Schema = wire::Schema::Dyn;
static S_MEASURED_OUT: wire::Schema = wire::Schema::Dyn;

/// `measured` over the binary wire.
#[wasm_bindgen(js_name = "measuredWire")]
pub fn measured_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let h: Value = wire::read_value(r, &S_MEASURED_0)?;
        let out = &port::measured(h)?;
        wire::write_value(w, out, &S_MEASURED_OUT)?;
        Ok(())
    })
}

static S_PREFERRED_OBSERVATION_0: wire::Schema = wire::Schema::Dyn;
static S_PREFERRED_OBSERVATION_OUT: wire::Schema = wire::Schema::Dyn;

/// `preferredObservation` over the binary wire.
#[wasm_bindgen(js_name = "preferredObservationWire")]
pub fn preferred_observation_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let health: Value = wire::read_value(r, &S_PREFERRED_OBSERVATION_0)?;
        let out = &port::preferred_observation(health)?;
        wire::write_value(w, out, &S_PREFERRED_OBSERVATION_OUT)?;
        Ok(())
    })
}

static S_FOLD_FAMILIES_0: wire::Schema = wire::Schema::Dyn;
static S_FOLD_FAMILIES_OUT: wire::Schema = wire::Schema::Dyn;

/// `foldFamilies` over the binary wire.
#[wasm_bindgen(js_name = "foldFamiliesWire")]
pub fn fold_families_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let rows: Value = wire::read_value(r, &S_FOLD_FAMILIES_0)?;
        let out = &port::fold_families(rows)?;
        wire::write_value(w, out, &S_FOLD_FAMILIES_OUT)?;
        Ok(())
    })
}

static S_PREFERRED_HEALTH_0: wire::Schema = wire::Schema::Dyn;
static S_PREFERRED_HEALTH_OUT: wire::Schema = wire::Schema::Dyn;

/// `preferredHealth` over the binary wire.
#[wasm_bindgen(js_name = "preferredHealthWire")]
pub fn preferred_health_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let node: Value = wire::read_value(r, &S_PREFERRED_HEALTH_0)?;
        let out = &port::preferred_health(node)?;
        wire::write_value(w, out, &S_PREFERRED_HEALTH_OUT)?;
        Ok(())
    })
}

static S_PROBE_KINDS_0: wire::Schema = wire::Schema::Dyn;
static S_PROBE_KINDS_OUT: wire::Schema = wire::Schema::Dyn;

/// `probeKinds` over the binary wire.
#[wasm_bindgen(js_name = "probeKindsWire")]
pub fn probe_kinds_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let health: Value = wire::read_value(r, &S_PROBE_KINDS_0)?;
        let out = &port::probe_kinds(health)?;
        wire::write_value(w, out, &S_PROBE_KINDS_OUT)?;
        Ok(())
    })
}

static S_HEALTH_MILLIS_OUT: wire::Schema = wire::Schema::Dyn;

/// `healthMillis` over the binary wire.
#[wasm_bindgen(js_name = "healthMillisWire")]
pub fn health_millis_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let health: Option<Value> = wire::read_dyn_opt(r)?;
        let out = &port::health_millis(health)?;
        wire::write_value(w, out, &S_HEALTH_MILLIS_OUT)?;
        Ok(())
    })
}

static S_COMPARE_LATENCY_OUT: wire::Schema = wire::Schema::Dyn;

/// `compareLatency` over the binary wire.
#[wasm_bindgen(js_name = "compareLatencyWire")]
pub fn compare_latency_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let a: Option<f64> = r.opt_js(|r| r.f64())?;
        let b: Option<f64> = r.opt_js(|r| r.f64())?;
        let out = &port::compare_latency(a, b)?;
        wire::write_value(w, out, &S_COMPARE_LATENCY_OUT)?;
        Ok(())
    })
}

static S_OUTBOUND_USAGE_OUT: wire::Schema = wire::Schema::Dyn;

/// `outboundUsage` over the binary wire.
#[wasm_bindgen(js_name = "outboundUsageWire")]
pub fn outbound_usage_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let snapshot: Option<Value> = wire::read_dyn_opt(r)?;
        let out = &port::outbound_usage(snapshot)?;
        wire::write_value(w, out, &S_OUTBOUND_USAGE_OUT)?;
        Ok(())
    })
}

static S_SIMULATED_ADDRESS_0: wire::Schema = wire::Schema::Dyn;
static S_SIMULATED_ADDRESS_OUT: wire::Schema = wire::Schema::OptJs(&wire::Schema::Str16);

/// `simulatedAddress` over the binary wire.
#[wasm_bindgen(js_name = "simulatedAddressWire")]
pub fn simulated_address_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let result: Value = wire::read_value(r, &S_SIMULATED_ADDRESS_0)?;
        let out = &port::simulated_address(result)?;
        wire::write_value(w, out, &S_SIMULATED_ADDRESS_OUT)?;
        Ok(())
    })
}

static S_PSEUDO_OWNER_0: wire::Schema = wire::Schema::Dyn;

/// `pseudoOwner` over the binary wire.
#[wasm_bindgen(js_name = "pseudoOwnerWire")]
pub fn pseudo_owner_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let node: Value = wire::read_value(r, &S_PSEUDO_OWNER_0)?;
        let out = &port::pseudo_owner(node)?;
        w.string16(out);
        Ok(())
    })
}

static S_NO_NODE_SOURCES_0: wire::Schema = wire::Schema::Dyn;
static S_NO_NODE_SOURCES_1: wire::Schema = wire::Schema::Dyn;

/// `noNodeSources` over the binary wire.
#[wasm_bindgen(js_name = "noNodeSourcesWire")]
pub fn no_node_sources_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let providers: Value = wire::read_value(r, &S_NO_NODE_SOURCES_0)?;
        let nodes: Value = wire::read_value(r, &S_NO_NODE_SOURCES_1)?;
        let out = &port::no_node_sources(providers, nodes)?;
        w.bool(*out);
        Ok(())
    })
}

static S_PSEUDO_OWNER_ID_1: wire::Schema = wire::Schema::Dyn;

/// `pseudoOwnerId` over the binary wire.
#[wasm_bindgen(js_name = "pseudoOwnerIdWire")]
pub fn pseudo_owner_id_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let kind: String = r.string16()?;
        let providers: Value = wire::read_value(r, &S_PSEUDO_OWNER_ID_1)?;
        let out = &port::pseudo_owner_id(kind, providers)?;
        w.string16(out);
        Ok(())
    })
}

static S_NODE_OWNER_0: wire::Schema = wire::Schema::Dyn;
static S_NODE_OWNER_1: wire::Schema = wire::Schema::Dyn;
static S_NODE_OWNER_OUT: wire::Schema = wire::Schema::Dyn;

/// `nodeOwner` over the binary wire.
#[wasm_bindgen(js_name = "nodeOwnerWire")]
pub fn node_owner_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let node: Value = wire::read_value(r, &S_NODE_OWNER_0)?;
        let providers: Value = wire::read_value(r, &S_NODE_OWNER_1)?;
        let out = &port::node_owner(node, providers)?;
        wire::write_value(w, out, &S_NODE_OWNER_OUT)?;
        Ok(())
    })
}

static S_ADDRESS_PORT_OUT: wire::Schema = wire::Schema::Dyn;

/// `addressPort` over the binary wire.
#[wasm_bindgen(js_name = "addressPortWire")]
pub fn address_port_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let address: Option<String> = r.opt_js(|r| r.string16())?;
        let out = &port::address_port(address)?;
        wire::write_value(w, out, &S_ADDRESS_PORT_OUT)?;
        Ok(())
    })
}

static S_CHAIN_NAMES_OUT: wire::Schema = wire::Schema::Dyn;

/// `chainNames` over the binary wire.
#[wasm_bindgen(js_name = "chainNamesWire")]
pub fn chain_names_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let chain: Vec<String> = r.list(|r| r.string16())?;
        let names: Option<Value> = wire::read_dyn_opt(r)?;
        let out = &port::chain_names(chain, names)?;
        wire::write_value(w, out, &S_CHAIN_NAMES_OUT)?;
        Ok(())
    })
}

static S_NODE_SUBSCRIPTION_0: wire::Schema = wire::Schema::Dyn;
static S_NODE_SUBSCRIPTION_OUT: wire::Schema = wire::Schema::Dyn;

/// `nodeSubscription` over the binary wire.
#[wasm_bindgen(js_name = "nodeSubscriptionWire")]
pub fn node_subscription_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let row: Value = wire::read_value(r, &S_NODE_SUBSCRIPTION_0)?;
        let subscriptions: Option<Value> = wire::read_dyn_opt(r)?;
        let out = &port::node_subscription(row, subscriptions)?;
        wire::write_value(w, out, &S_NODE_SUBSCRIPTION_OUT)?;
        Ok(())
    })
}

static S_LIFECYCLE_TONE_OUT: wire::Schema = wire::Schema::Dyn;

/// `lifecycleTone` over the binary wire.
#[wasm_bindgen(js_name = "lifecycleToneWire")]
pub fn lifecycle_tone_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let state: Option<Value> = wire::read_dyn_opt(r)?;
        let out = &port::lifecycle_tone(state)?;
        wire::write_value(w, out, &S_LIFECYCLE_TONE_OUT)?;
        Ok(())
    })
}

static S_CLOSED_ALL_TONE_0: wire::Schema = wire::Schema::Dyn;
static S_CLOSED_ALL_TONE_OUT: wire::Schema = wire::Schema::Dyn;

/// `closedAllTone` over the binary wire.
#[wasm_bindgen(js_name = "closedAllToneWire")]
pub fn closed_all_tone_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let tally: Value = wire::read_value(r, &S_CLOSED_ALL_TONE_0)?;
        let out = &port::closed_all_tone(tally)?;
        wire::write_value(w, out, &S_CLOSED_ALL_TONE_OUT)?;
        Ok(())
    })
}

static S_CONNECTION_ROWS_OUT: wire::Schema = wire::Schema::Dyn;

/// `connectionRows` over the binary wire.
#[wasm_bindgen(js_name = "connectionRowsWire")]
pub fn connection_rows_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let snapshot: Option<Value> = wire::read_dyn_opt(r)?;
        let out = &port::connection_rows(snapshot)?;
        wire::write_value(w, out, &S_CONNECTION_ROWS_OUT)?;
        Ok(())
    })
}

static S_ROUTINE_GAP_0: wire::Schema = wire::Schema::Dyn;

/// `routineGap` over the binary wire.
#[wasm_bindgen(js_name = "routineGapWire")]
pub fn routine_gap_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let event: Value = wire::read_value(r, &S_ROUTINE_GAP_0)?;
        let out = &port::routine_gap(event)?;
        w.bool(*out);
        Ok(())
    })
}

static S_SHORT_ID_OUT: wire::Schema = wire::Schema::Dyn;

/// `shortId` over the binary wire.
#[wasm_bindgen(js_name = "shortIdWire")]
pub fn short_id_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let id: String = r.string16()?;
        let out = &port::short_id(id)?;
        wire::write_value(w, out, &S_SHORT_ID_OUT)?;
        Ok(())
    })
}

static S_RESOLVE_SELECTED_LEAF_2: wire::Schema = wire::Schema::Dyn;
static S_RESOLVE_SELECTED_LEAF_3: wire::Schema = wire::Schema::Dyn;
static S_RESOLVE_SELECTED_LEAF_4: wire::Schema = wire::Schema::Dyn;
static S_RESOLVE_SELECTED_LEAF_OUT: wire::Schema = wire::Schema::Dyn;

/// `resolveSelectedLeaf` over the binary wire.
#[wasm_bindgen(js_name = "resolveSelectedLeafWire")]
pub fn resolve_selected_leaf_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let outbound: String = r.string16()?;
        let network: String = r.string16()?;
        let groups_by_name: Value = wire::read_value(r, &S_RESOLVE_SELECTED_LEAF_2)?;
        let groups_by_id: Value = wire::read_value(r, &S_RESOLVE_SELECTED_LEAF_3)?;
        let nodes_by_id: Value = wire::read_value(r, &S_RESOLVE_SELECTED_LEAF_4)?;
        let out = &port::resolve_selected_leaf(
            outbound,
            network,
            groups_by_name,
            groups_by_id,
            nodes_by_id,
        )?;
        wire::write_value(w, out, &S_RESOLVE_SELECTED_LEAF_OUT)?;
        Ok(())
    })
}

static S_INTERESTING_NOTICE_0: wire::Schema = wire::Schema::Dyn;
static S_INTERESTING_NOTICE_OUT: wire::Schema = wire::Schema::Dyn;

/// `interestingNotice` over the binary wire.
#[wasm_bindgen(js_name = "interestingNoticeWire")]
pub fn interesting_notice_wire(len: u32) -> Result<u32, JsError> {
    wire_io(len, |r, w| {
        let event: Value = wire::read_value(r, &S_INTERESTING_NOTICE_0)?;
        let out = &port::interesting_notice(event)?;
        wire::write_value(w, out, &S_INTERESTING_NOTICE_OUT)?;
        Ok(())
    })
}
