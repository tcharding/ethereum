//! Test the geth RPC client. To run these tests access to a geth node is
//! required.
use std::str::FromStr;

use anyhow::Result;

use ethereum::geth::jsonrpc_reqwest::{Client, Url};

// URL of the geth node to test against.
const GETH_URL: &str = "http://localhost:8545/";

fn client() -> Client {
    let url = Url::from_str(GETH_URL).expect("failed to parse url");
    Client::new(url)
}

#[tokio::test]
async fn can_connect_to_geth_node() -> Result<()> {
    let cli = client();
    let _ = cli.client_version().await?;
    Ok(())
}
