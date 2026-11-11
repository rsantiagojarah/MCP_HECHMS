#[tokio::main]
async fn main() -> anyhow::Result<()> {
    mcp_hechms::run().await
}
