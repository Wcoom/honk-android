//! The Rust port of `src/api/rings.ts`.

#![forbid(unsafe_code)]

/// Port of `mean` (`src/api/rings.ts:13`).
pub fn mean(values: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    let mut sum = 0.0_f64;
    let mut count = 0usize;
    for value in &values {
        if value.is_null() {
            continue;
        }
        count += 1;
        let n = crate::js::as_number(value).unwrap_or_else(|| match value {
            serde_json::Value::Bool(b) => f64::from(*b),
            _ => f64::NAN,
        });
        sum += n;
    }
    if count == 0 {
        Ok(serde_json::Value::Null)
    } else {
        Ok(crate::js::number(sum / count as f64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn empty_and_all_null_return_null() {
        assert_eq!(mean(vec![]).unwrap(), Value::Null);
        assert_eq!(mean(vec![Value::Null, Value::Null]).unwrap(), Value::Null);
    }

    #[test]
    fn averages_known_values_skipping_nulls() {
        assert_eq!(
            mean(vec![json!(2), Value::Null, json!(6)]).unwrap(),
            json!(4)
        );
        assert_eq!(mean(vec![json!(-4), json!(1)]).unwrap(), json!(-1.5));
        assert_eq!(mean(vec![json!(1), json!(2), json!(0)]).unwrap(), json!(1));
    }

    #[test]
    fn preserves_js_special_numbers_and_undefined() {
        assert_eq!(
            mean(vec![crate::js::undefined(), json!(5)]).unwrap(),
            json!({ "$num": "NaN" })
        );
        assert_eq!(
            mean(vec![json!({ "$num": "NaN" }), json!(1)]).unwrap(),
            json!({ "$num": "NaN" })
        );
        assert_eq!(
            mean(vec![json!({ "$num": "Infinity" }), json!(5)]).unwrap(),
            json!({ "$num": "Infinity" })
        );
        assert_eq!(mean(vec![json!({ "$num": "-0" })]).unwrap(), json!(0));
    }
}
