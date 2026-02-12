use crate::routes::{ping, echo, add};
use serde_json::json;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

pub fn route(req: JsonRpcRequest) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "ping" => ping::handle(req.params),
        "echo" => echo::handle(req.params),
        "add" => add::handle(req.params),
        _ => json!({"error": "Unknown method"}),
    };
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id: req.id,
    }
}
