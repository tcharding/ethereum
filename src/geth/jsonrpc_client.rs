//! JSON RPC client for go-ethereum, uses `reqwest` by way of the
//! `jsonrpc_client` library. ref: https://eth.wiki/json-rpc/API
use std::convert::TryFrom;
use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use clarity::Uint256;
use jsonrpc_client::implement;
pub use jsonrpc_client::Url;

use crate::geth::{BlockNumber, GethClientAsync};
use crate::types::{CallRequest, TransactionReceipt, H256};
use crate::{Address, ChainId, Ether, Gwei, Wei};

#[jsonrpc_client::api(version = "1.0")]
trait GethRpc {
    #[allow(non_snake_case)]
    async fn web3_clientVersion(&self) -> String;
    #[allow(non_snake_case)]
    async fn net_version(&self) -> String;
    #[allow(non_snake_case)]
    async fn eth_sendRawTransaction(&self, txn_hex: String) -> H256;
    #[allow(non_snake_case)]
    async fn eth_getTransactionReceipt(&self, txn: H256) -> Option<TransactionReceipt>;
    #[allow(non_snake_case)]
    async fn eth_getTransactionCount(&self, account: Address, height: BlockNumber) -> u32;
    #[allow(non_snake_case)]
    async fn eth_getBalance(&self, account: Address, height: BlockNumber) -> String;
    #[allow(non_snake_case)]
    async fn eth_gasPrice(&self) -> String;
    #[allow(non_snake_case)]
    async fn eth_estimateGas(&self, request: CallRequest, height: BlockNumber) -> String;
}

#[implement(GethRpc)]
pub struct Client {
    inner: reqwest::Client,
    base_url: Url,
}

#[async_trait]
impl GethClientAsync for Client {
    fn new(base_url: Url) -> Self {
        Self {
            inner: reqwest::Client::new(),
            base_url,
        }
    }

    async fn client_version(&self) -> Result<String> {
        let version = self.web3_clientVersion().await?;
        Ok(version)
    }

    async fn chain_id(&self) -> Result<ChainId> {
        let version = self.net_version().await?;
        let chain_id = ChainId::try_from(version)?;

        Ok(chain_id)
    }

    async fn send_raw_transaction(&self, transaction_hex: String) -> Result<H256> {
        let hash = self.eth_sendRawTransaction(transaction_hex).await?;
        Ok(hash)
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> Result<Option<TransactionReceipt>> {
        let receipt = self.eth_getTransactionReceipt(transaction_hash).await?;
        Ok(receipt)
    }

    async fn get_transaction_count(&self, account: Address, height: BlockNumber) -> Result<u32> {
        let count = self.eth_getTransactionCount(account, height).await?;
        Ok(count)
    }

    async fn get_balance(&self, account: Address, height: BlockNumber) -> Result<Ether> {
        let hex = self.eth_getBalance(account, height).await?;
        let balance = Wei::try_from_hex_str(&hex)?;
        Ok(balance.into())
    }

    async fn gas_price(&self) -> Result<Gwei> {
        let hex = self.eth_gasPrice().await?;
        let gas = Wei::try_from_hex_str(&hex)?;
        Ok(gas.into())
    }

    async fn gas_limit(&self, request: CallRequest, height: BlockNumber) -> Result<Uint256> {
        let hex = self.eth_estimateGas(request, height).await?;
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
