use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::channel::BasicPublishArguments;
use amqprs::BasicProperties;
use anyhow::Context;
use tokio;
use serde::{Serialize};
use serde_json::json;

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .with_context(|| "Failed to connect to RabbitMQ")?;

    let channel = conn
        .open_channel(None)
        .await
        .with_context(|| "Failed to open channel")?;

    let reply_to_queue = "rpc_reply_queue";

    let requests = vec![
        JsonRpcRequest { jsonrpc: "2.0".to_string(), method: "ping".to_string(), params: None, id: Some(json!(1)) },
        JsonRpcRequest { jsonrpc: "2.0".to_string(), method: "echo".to_string(), params: Some(json!("Hello RPC!")), id: Some(json!(2)) },
        JsonRpcRequest { jsonrpc: "2.0".to_string(), method: "add".to_string(), params: Some(json!({"a": 3, "b": 7})), id: Some(json!(3)) },
    ];

    for req in requests {
        let payload = serde_json::to_vec(&req)?;

        let mut properties = BasicProperties::default();
        properties.with_reply_to(reply_to_queue);

        let publish_args = BasicPublishArguments::new("my_exchange", "rpc");

        channel.basic_publish(properties, payload, publish_args).await?;

        println!("Sent {} request", req.method);
    }

    Ok(())
}
