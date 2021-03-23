use anyhow::Result;

use ethereum::geth::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Client::localhost();

    let version = cli.client_version().await?;
    let id = cli.chain_id().await?;
    println!(
        "Connected to local geth instance: \n  version: {}\n  chain id: {}",
        version, id
    );

    Ok(())
}
