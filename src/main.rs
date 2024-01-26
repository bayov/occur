#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_backtrace::install();

    let _ = scylla::SessionBuilder::new()
        .known_node("127.0.0.1:9042")
        .build()
        .await?;

    Ok(())
}
