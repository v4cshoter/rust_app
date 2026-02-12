use serde_json::{json, Value};

pub fn handle(params: Option<Value>) -> Value {
    if let Some(p) = params {
        let a = p.get("a").and_then(|v| v.as_i64()).unwrap_or(0);
        let b = p.get("b").and_then(|v| v.as_i64()).unwrap_or(0);
        json!(a + b)
    } else {
        json!(0)
    }
}
