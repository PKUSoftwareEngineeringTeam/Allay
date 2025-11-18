use allay_cli::execute;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    execute().await
}
