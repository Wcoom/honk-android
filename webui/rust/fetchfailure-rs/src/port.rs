//! The Rust port of `apps/lite/src/fetchFailure.ts`.

#![forbid(unsafe_code)]

use serde_json::Value;

fn read_prop(obj: &Value, key: &str) -> Result<Value, String> {
    if obj.is_null() {
        return Err(format!(
            "TypeError: Cannot read properties of null (reading '{key}')"
        ));
    }
    if crate::js::is_undefined(obj) {
        return Err(format!(
            "TypeError: Cannot read properties of undefined (reading '{key}')"
        ));
    }
    Ok(obj
        .as_object()
        .and_then(|map| map.get(key))
        .cloned()
        .unwrap_or_else(crate::js::undefined))
}

/// Port of `fetchFailureText` (`apps/lite/src/fetchFailure.ts:18`).
pub fn fetch_failure_text(
    error: serde_json::Value,
    t: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let code = read_prop(&error, "code")?;
    if code.as_str() == Some("route_unavailable") {
        return read_prop(&t, "routeUnavailable");
    }
    if code.as_str() != Some("fetch_failed") {
        return read_prop(&error, "message");
    }
    let details = read_prop(&error, "details")?;
    let stage = if details.is_null() || crate::js::is_undefined(&details) {
        crate::js::undefined()
    } else {
        read_prop(&details, "stage")?
    };
    let key = match stage.as_str() {
        Some("connection_failed") => "fetchStageConnection",
        Some("download_timeout") => "fetchStageTimeout",
        Some("tls_failed") => "fetchStageTls",
        Some("http_status_rejected") => "fetchStageHttpStatus",
        Some("http_not_found") => "fetchStageNotFound",
        Some("route_blocked") => "fetchStageRouteBlocked",
        Some("asset_too_large") => "fetchStageTooLarge",
        Some("parse_failed") => "fetchStageParse",
        _ => "fetchFailed",
    };
    read_prop(&t, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_known_stages_and_fallbacks() {
        let t = json!({
            "routeUnavailable": "no route",
            "fetchFailed": "plain failed",
            "fetchStageConnection": "conn",
            "fetchStageTimeout": "timeout",
            "fetchStageTls": "tls",
            "fetchStageHttpStatus": "status",
            "fetchStageNotFound": "404",
            "fetchStageRouteBlocked": "blocked",
            "fetchStageTooLarge": "large",
            "fetchStageParse": "parse",
        });
        assert_eq!(
            fetch_failure_text(json!({"code": "route_unavailable"}), t.clone()).unwrap(),
            json!("no route")
        );
        assert_eq!(
            fetch_failure_text(json!({"code": "other", "message": "raw"}), t.clone()).unwrap(),
            json!("raw")
        );
        assert_eq!(
            fetch_failure_text(
                json!({"code": "fetch_failed", "details": {"stage": "parse_failed"}}),
                t.clone()
            )
            .unwrap(),
            json!("parse")
        );
        assert_eq!(
            fetch_failure_text(
                json!({"code": "fetch_failed", "details": {"stage": "toString"}}),
                t.clone()
            )
            .unwrap(),
            json!("plain failed")
        );
        assert_eq!(
            fetch_failure_text(json!({"code": "fetch_failed", "details": null}), t).unwrap(),
            json!("plain failed")
        );
    }

    #[test]
    fn handles_null_and_primitive_inputs_like_js() {
        assert!(fetch_failure_text(Value::Null, json!({})).is_err());
        let res = fetch_failure_text(json!(0), json!(0)).unwrap();
        assert!(crate::js::is_undefined(&res));
    }
}
