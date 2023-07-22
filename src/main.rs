use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    jokolay::start_jokolay().await;
    Ok(())
}
