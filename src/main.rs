use tokio;
use rust_app::rabbitmq;

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() -> anyhow::Result<()> {
    rabbitmq::consumer::start().await?;
    Ok(())
}
