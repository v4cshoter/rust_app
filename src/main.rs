use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, 
    Channel, ExchangeType, ExchangeDeclareArguments, 
    QueueDeclareArguments, QueueBindArguments};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use tokio;
use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

#[async_trait]
impl AsyncConsumer for RpcConsumer {
    async fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let message = String::from_utf8_lossy(&content);
        println!("Received message: {}", message);

        if let Some(reply_to) = properties.reply_to() {
            println!("Reply_to queue: {}", reply_to);

            let result = self.channel.basic_publish(
                BasicProperties::default(),
                content.clone(),
                BasicPublishArguments {
                    routing_key: reply_to.clone(),
                    ..BasicPublishArguments::default()
                },
            )
            .await;

            match result {
                Ok(_) => println!("Message sent back to {}", reply_to),
                Err(e) => eprintln!("Failed to send message: {}", e),
            }
        } else {
            println!("No reply_to header");
        }
    }
}
