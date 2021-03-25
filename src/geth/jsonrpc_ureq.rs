//! JSON RPC client for go-ethereum, uses `ureq` by way of `../jsonrpc_ureq`.
//! ref: https://eth.wiki/json-rpc/API
use std::str::FromStr;

use anyhow::{Context, Result};
use clarity::Uint256;
use ethereum_types::U256;
pub use url::Url;

use crate::{
    jsonrpc_ureq, Address, Amount, ChainId, Erc20, Ether, Hash, TransactionReceipt, UnformattedData,
};

#[derive(Debug, Clone)]
pub struct Client {
    inner: jsonrpc_async::Client,
}

impl Client {
    pub fn new(url: Url) -> Self {
        Client {
            inner: jsonrpc_async::Client::new(url),
        }
    }

    pub fn localhost() -> Result<Self> {
        let url = Url::from_str("http://localhost:8545/")?;
        let client = Client::new(url);

        Ok(client)
    }

    /// Execute RPC method: `web3_clientVersion`. Return version string:
    /// "Geth/v1.10.2-unstable-f304290b-20210323/linux-amd64/go1.13.8"
    pub async fn client_version(&self) -> Result<String> {
        let version = self
            .inner
            .send::<Vec<()>, String>(jsonrpc_async::Request::v2("web3_clientVersion", vec![]))
            .await
            .context("failed to fetch client version")?;

        Ok(version)
    }
}
