use serde_json::{json, Value};

pub fn handle(_params: Option<Value>) -> Value {
    json!("pong")
}