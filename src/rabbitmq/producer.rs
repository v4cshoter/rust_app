use amqprs::BasicProperties;
use amqprs::channel::BasicPublishArguments;
use amqprs::connection::{Connection, OpenConnectionArguments};
use serde_json::Value;
use anyhow::Context;
use crate::routing::JsonRpcRequest;

pub async fn produce(method: &str, params: Option<Value>, id: Option<serde_json::Value>) -> anyhow::Result<()> {
    let conn = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .with_context(|| "Failed to connect")?;

    let channel = conn.open_channel(None).await?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id,
    };

    let payload = serde_json::to_vec(&request)?;

    let mut properties = BasicProperties::default();
    properties.with_reply_to("rpc_reply_queue");

    let publish_args = BasicPublishArguments::new("my_exchange", "rpc");

    channel.basic_publish(properties, payload, publish_args).await?;

    Ok(())
}
