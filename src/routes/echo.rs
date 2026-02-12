use serde_json::{json, Value};

pub fn handle(params: Option<Value>) -> Value {
    params.unwrap_or(json!(null))
}