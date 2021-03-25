use anyhow::Result;

use ethereum::geth_async::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Client::localhost()?;

    let _version = cli.client_version().await?;

    let chain = cli.chain_id().await?;
    println!("{}", chain);

    Ok(())
}
