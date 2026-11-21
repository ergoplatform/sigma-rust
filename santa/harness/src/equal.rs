//! The structural equality of a grading comparison (docs/contract/grading.md §3). A copy of SANTA's
//! tools/santa-check/src/lib.rs (structural_equal, num_eq); keep it in step with that file.
use serde_json::{Number, Value};

/// §5 structural equality: objects key-order-insensitive, arrays order-sensitive, numbers numeric,
/// strings exact, null == only null, bool is NOT int.
pub fn structural_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Bool(_), _) | (_, Value::Bool(_)) => false, // bool only equals bool
        (Value::Null, Value::Null) => true,
        (Value::Null, _) | (_, Value::Null) => false, // null only equals null
        (Value::Number(x), Value::Number(y)) => num_eq(x, y),
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| structural_equal(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| structural_equal(v, w)))
        }
        _ => false,
    }
}

/// Numeric equality across int/float reprs (mirrors Python ==), exact for the i64/u64 the corpus uses.
fn num_eq(x: &Number, y: &Number) -> bool {
    if let (Some(a), Some(b)) = (x.as_i64(), y.as_i64()) {
        return a == b;
    }
    if let (Some(a), Some(b)) = (x.as_u64(), y.as_u64()) {
        return a == b;
    }
    matches!((x.as_f64(), y.as_f64()), (Some(a), Some(b)) if a == b)
}
