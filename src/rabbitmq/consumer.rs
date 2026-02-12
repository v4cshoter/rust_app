use crate::routing;
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::channel::{
    BasicConsumeArguments, 
    Channel, ExchangeType, ExchangeDeclareArguments, 
    QueueDeclareArguments, QueueBindArguments};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use anyhow::Context;
use serde_json::json;

pub async fn start() -> anyhow::Result<()> {
    // Подключение к rabbit
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

    channel
        .exchange_declare(ExchangeDeclareArguments {
            exchange: "my_exchange".to_string(),
            exchange_type: ExchangeType::Direct.to_string(),
            ..Default::default()
            }
        )
        .await
        .with_context(|| "failed to declare exchange")?;

    channel
        .queue_declare(QueueDeclareArguments::new("rpc_request_queue"))
        .await
        .with_context(|| "Failed to declare queue")?
        .unwrap();

    channel.queue_bind(
        QueueBindArguments::new(
            "rpc_request_queue",
            "my_exchange",
            "rpc",
        )
    ).await?;

    channel
        .basic_consume(
            RpcConsumer { channel: channel.clone() },
            BasicConsumeArguments::new("rpc_request_queue", "my_consumer"),
        )
        .await
        .context("failed to start consuming from rpc_request_queue")?;

    println!("Waiting for messages...");

    tokio::signal::ctrl_c().await?;
    println!("Exiting");
    Ok(())
}

struct RpcConsumer {
    channel: Channel,
}
#[async_trait::async_trait]
impl AsyncConsumer for RpcConsumer {
    async fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let message = String::from_utf8_lossy(&content);
        let request: serde_json::Result<crate::routing::JsonRpcRequest> = serde_json::from_str(&message);
        

        let response = match request {
            Ok(req) => routing::route(req),
            Err(e) => crate::routing::JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(json!({"code": -32700, "message": e.to_string()})),
                id: None,
            },
        };

        if let Some(reply_to) = properties.reply_to() {
            let response_bytes = serde_json::to_vec(&response).unwrap_or_default();
            let publish_args = amqprs::channel::BasicPublishArguments {
                routing_key: reply_to.clone(),
                ..Default::default()
            };
            self.channel
                .basic_publish(BasicProperties::default(), response_bytes, publish_args)
                .await
                .unwrap();
        }
    }
}