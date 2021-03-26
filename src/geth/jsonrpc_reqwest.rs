//! JSON RPC client for go-ethereum, uses `reqwest` by way of
//! `../jsonrpc_reqwest`. ref: https://eth.wiki/json-rpc/API
use anyhow::{Context, Result};
use async_trait::async_trait;
use clarity::Uint256;

use crate::geth::GethClientAsync;
pub use crate::jsonrpc_reqwest::Url;
use crate::types::{BlockNumber, CallRequest, TransactionReceipt, H256};
use crate::{Address, ChainId, Ether, Gwei, Wei};

use crate::jsonrpc_reqwest as rpc;

#[derive(Debug, Clone)]
pub struct Client {
    inner: rpc::Client,
}

#[async_trait]
impl GethClientAsync for Client {
    fn new(url: Url) -> Self {
        Client {
            inner: rpc::Client::new(url),
        }
    }

    /// Execute RPC method: `web3_clientVersion`. Return version string:
    /// "Geth/v1.10.2-unstable-f304290b-20210323/linux-amd64/go1.13.8"
    async fn client_version(&self) -> Result<String> {
        let version = self
            .inner
            .send::<Vec<()>, String>(rpc::Request::v2("web3_clientVersion", vec![]))
            .await
            .context("failed to fetch client version")?;

        Ok(version)
    }

    /// Execute RPC method: `net_version`. Return network id (chain id).
    async fn chain_id(&self) -> Result<ChainId> {
        let chain_id = self
            .inner
            .send::<Vec<()>, String>(rpc::Request::v2("net_version", vec![]))
            .await
            .context("failed to fetch net version")?;
        let chain_id: u32 = chain_id.parse()?;
        let chain_id = ChainId::from(chain_id);

        Ok(chain_id)
    }

    /// Execute RPC method: `eth_sendRawTransaction`. Return transaction hash.
    async fn send_raw_transaction(&self, transaction_hex: String) -> Result<H256> {
        let tx_hash = self
            .inner
            .send(rpc::Request::v2("eth_sendRawTransaction", vec![
                transaction_hex,
            ]))
            .await
            .context("failed to send raw transaction")?;

        Ok(tx_hash)
    }

    /// Execute RPC method: `eth_getTransactionReceipt`.
    async fn get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> Result<Option<TransactionReceipt>> {
        let receipt = self
            .inner
            .send(rpc::Request::v2("eth_getTransactionReceipt", vec![
                rpc::serialize(transaction_hash)?,
            ]))
            .await
            .context("failed to get transaction receipt")?;

        Ok(receipt)
    }

    /// Execute RPC method: `eth_getTransactionCount`. Return the number of
    /// transactions sent from this address.
    async fn get_transaction_count(&self, account: Address, height: BlockNumber) -> Result<u32> {
        let count: String = self
            .inner
            .send(rpc::Request::v2("eth_getTransactionCount", vec![
                rpc::serialize(account)?,
                rpc::serialize(height)?,
            ]))
            .await
            .context("failed to get transaction count")?;

        let count = u32::from_str_radix(&count[2..], 16)?;
        Ok(count)
    }

    async fn get_balance(&self, address: Address, height: BlockNumber) -> Result<Ether> {
        let amount: String = self
            .inner
            .send(rpc::Request::v2("eth_getBalance", vec![
                rpc::serialize(address)?,
                rpc::serialize(height)?,
            ]))
            .await
            .context("failed to get balance")?;
        let wei = Wei::try_from_hex_str(&amount)?;

        Ok(wei.into())
    }

    async fn gas_price(&self) -> Result<Gwei> {
        let amount = self
            .inner
            .send::<Vec<()>, String>(rpc::Request::v2("eth_gasPrice", vec![]))
            .await
            .context("failed to get gas price")?;
        let amount = Wei::try_from_hex_str(&amount[2..])?;

        Ok(amount.into())
    }

    async fn gas_limit(&self, request: CallRequest, height: BlockNumber) -> Result<Uint256> {
        let gas_limit: String = self
            .inner
            .send(rpc::Request::v2("eth_estimateGas", vec![
                rpc::serialize(request)?,
                rpc::serialize(height)?,
            ]))
            .await
            .context("failed to get gas price")?;
        let gas_limit = Uint256::from_str_radix(&gas_limit[2..], 16)?;

        Ok(gas_limit)
    }
}
