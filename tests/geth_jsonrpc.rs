//! Test the geth RPC client. To run these tests access to a geth node is
//! required.
use anyhow::Result;

use ethereum::geth::jsonrpc_client::Client;

// URL of the geth node to test against.
const GETH_URL: &str = "http://localhost:8545/";
// `geth -networkid <CHAIN_ID>`

#[tokio::test]
async fn can_connect_to_geth_node() -> Result<()> {
    let cli = Client::new(GETH_URL)?;
    let _ = cli.client_version().await?;
    Ok(())
}
