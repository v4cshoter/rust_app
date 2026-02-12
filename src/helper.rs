use tokio;
use rust_app::rabbitmq;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let produce = crate::rabbitmq::producer::produce;

    produce("ping", None, Some(json!(1))).await?;
    produce("echo", Some(json!({"msg": "hello"})), Some(json!(2))).await?;
    produce("add", Some(json!({"a": 4, "b": 4})), Some(json!(3))).await?;

    Ok(())
}
