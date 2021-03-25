//! JSON RPC client for go-ethereum, uses `reqwest` by way of the
//! `jsonrpc_client` library. ref: https://eth.wiki/json-rpc/API
use std::convert::TryFrom;
use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;

use anyhow::Result;
use clarity::Uint256;
use ethereum_types::U256;
use jsonrpc_client::{implement, Url};

use crate::geth::DefaultBlock;
use crate::{Address, Amount, ChainId, Ether, Hash, TransactionReceipt};

#[jsonrpc_client::api(version = "1.0")]
trait GethRpc {
    #[allow(non_snake_case)]
    async fn web3_clientVersion(&self) -> String;
    #[allow(non_snake_case)]
    async fn net_version(&self) -> String;
    #[allow(non_snake_case)]
    async fn eth_sendRawTransaction(&self, txn_hex: String) -> Hash;
    #[allow(non_snake_case)]
    async fn eth_getTransactionReceipt(&self) -> TransactionReceipt;
    #[allow(non_snake_case)]
    async fn eth_getTransactionCount(&self, account: Address) -> u32;
    #[allow(non_snake_case)]
    async fn eth_getBalance(&self, account: Address, height: String) -> String;
    #[allow(non_snake_case)]
    async fn eth_gasPrice(&self) -> String;
    #[allow(non_snake_case)]
    async fn eth_estimateGas(&self) -> String;
}

#[implement(GethRpc)]
pub struct Client {
    inner: reqwest::Client,
    base_url: Url,
}

impl Client {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            base_url: base_url.parse()?,
        })
    }

    pub fn localhost() -> Result<Self> {
        Client::new("http://localhost:8545/")
    }

    pub async fn client_version(&self) -> Result<String> {
        let version = self.web3_clientVersion().await?;
        Ok(version)
    }

    pub async fn chain_id(&self) -> Result<ChainId> {
        let version = self.net_version().await?;
        let chain_id = ChainId::try_from(version)?;

        Ok(chain_id)
    }

    pub async fn send_raw_transaciton(&self, transaction_hex: String) -> Result<Hash> {
        let hash = self.eth_sendRawTransaction(transaction_hex).await?;
        Ok(hash)
    }

    pub async fn get_transaciton_receipt(&self) -> Result<TransactionReceipt> {
        let receipt = self.eth_getTransactionReceipt().await?;
        Ok(receipt)
    }

    pub async fn get_transaction_count(&self, account: Address) -> Result<u32> {
        let count = self.eth_getTransactionCount(account).await?;
        Ok(count)
    }

    pub async fn get_balance(&self, account: Address, height: DefaultBlock) -> Result<Amount> {
        let hex = self.eth_getBalance(account, height.to_string()).await?;
        let balance = Amount::try_from_hex_str(&hex)?;
        Ok(balance)
    }

    pub async fn gas_price(&self) -> Result<Ether> {
        let hex = self.eth_gasPrice().await?;
        let gas = Ether::try_from_hex_str(&hex)?;
        Ok(gas)
    }

    pub async fn estimate_gas(&self) -> Result<Uint256> {
        let hex = self.eth_estimateGas().await?;
        let gas = Uint256::from_str(&hex)?;
        Ok(gas)
    }
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("inner", &self.inner)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct EstimateGasRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<Uint256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_default_block_num() {
        let height = DefaultBlock::Num(1234);
        let want = "0x4d2";
        let got = format!("{}", height);

        assert_eq!(got, want);
    }
}
