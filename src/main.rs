use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::channel::{Channel, BasicPublishArguments, BasicConsumeArguments};
use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use tokio;

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
    .unwrap();
    conn
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();
    println!("Connected to RabbitMQ");

    let channel = conn.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();
    println!("Channel opened");

    channel
        .basic_consume(
            RpcConsumer { channel: channel.clone() },
            BasicConsumeArguments::new("rpc_request_queue", "my_consumer"),
        )
        .await?;
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
