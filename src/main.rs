use tokio;
use rust_app::rabbitmq;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rabbitmq::consumer::start().await?;
    Ok(())
}