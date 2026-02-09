use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::channel::{BasicPublishArguments};
use amqprs::BasicProperties;
use anyhow::Context;
use tokio;

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

    let payload = b"Hello from producer!".to_vec();
    let reply_to_queue = "rpc_reply_queue";

    let publish_args = BasicPublishArguments::new("", "rpc_request_queue");

    let mut properties = BasicProperties::default();
    let properties = properties.with_reply_to(reply_to_queue).clone();
    channel.basic_publish(properties, payload, publish_args).await?;

    println!("Message sent to rpc_request_queue with reply_to={}", reply_to_queue);

    Ok(())
}
