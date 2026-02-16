use crate::routing;
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::channel::{
    BasicConsumeArguments, Channel, ExchangeType, ExchangeDeclareArguments,
    QueueDeclareArguments, QueueBindArguments, BasicPublishArguments,
    BasicAckArguments, BasicQosArguments,
};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use anyhow::Context;
use serde_json::json;
use tokio::time::{sleep, Duration};
use tokio::sync::{Semaphore, Mutex};
use std::sync::Arc;

const MAX_CONCURRENT: u16 = 3;

pub async fn start() -> anyhow::Result<()> {
    let conn = Connection::open(&OpenConnectionArguments::new(
        "localhost", 5672, "guest", "guest"
    ))
    .await
    .with_context(|| "Failed to connect to RabbitMQ")?;

    let channel = conn
        .open_channel(None)
        .await
        .with_context(|| "Failed to open channel")?;

    channel
        .basic_qos(BasicQosArguments::new(0, MAX_CONCURRENT, false))
        .await?;

    channel
        .exchange_declare(ExchangeDeclareArguments {
            exchange: "my_exchange".to_string(),
            exchange_type: ExchangeType::Direct.to_string(),
            ..Default::default()
        })
        .await?;

    channel.queue_declare(QueueDeclareArguments::new("rpc_reply_queue")).await?.unwrap();
    channel.queue_declare(QueueDeclareArguments::new("rpc_request_queue")).await?.unwrap();

    channel.queue_bind(QueueBindArguments::new(
        "rpc_request_queue",
        "my_exchange",
        "rpc",
    )).await?;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT as usize));
    let counter = Arc::new(Mutex::new(0u64));

    channel
        .basic_consume(
            RpcConsumer {
                channel: channel.clone(),
                semaphore,
                counter,
            },
            BasicConsumeArguments::new("rpc_request_queue", "my_consumer"),
        )
        .await?;

    println!("Waiting for messages...");

    tokio::signal::ctrl_c().await?;
    Ok(())
}

struct RpcConsumer {
    channel: Channel,
    semaphore: Arc<Semaphore>,
    counter: Arc<Mutex<u64>>,
}

#[async_trait::async_trait]
impl AsyncConsumer for RpcConsumer {
    async fn consume(
        &mut self,
        _channel: &Channel,
        deliver: Deliver,
        properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let semaphore = self.semaphore.clone();
        let channel = self.channel.clone();
        let counter = self.counter.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.unwrap();

            let message = String::from_utf8_lossy(&content);
            println!("Processing message: {}", message);

            sleep(Duration::from_secs(3)).await;

            {
                let mut count = counter.lock().await;
                *count += 1;
                println!("Total processed: {}", *count);
            }

            let request: serde_json::Result<crate::routing::JsonRpcRequest> =
                serde_json::from_str(&message);

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
                let publish_args = BasicPublishArguments {
                    routing_key: reply_to.clone(),
                    ..Default::default()
                };

                if let Err(e) = channel
                    .basic_publish(BasicProperties::default(), response_bytes, publish_args)
                    .await
                {
                    eprintln!("Publish error: {:?}", e);
                }
            }

            let ack_args = BasicAckArguments::new(deliver.delivery_tag(), false);

            match channel.basic_ack(ack_args).await {
                Ok(_) => println!("Message acknowledged: {}", message),
                Err(err) => eprintln!("Ack error: {:?}", err),
            }
        });
    }
}
