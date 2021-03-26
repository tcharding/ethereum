//! JSON RPC client for go-ethereum, uses `ureq` by way of `../jsonrpc_ureq`.
//! ref: https://eth.wiki/json-rpc/API

use anyhow::{Context, Result};
use clarity::Uint256;

use crate::geth::BlockNumber;
use crate::geth::GethClient;
pub use crate::jsonrpc_ureq::Url;
use crate::types::{CallRequest, TransactionReceipt, H256};
use crate::{Address, ChainId, Ether, Wei};

use crate::jsonrpc_ureq as rpc;

#[derive(Debug, Clone)]
pub struct Client {
    inner: rpc::Client,
}

impl GethClient for Client {
    fn new(base_url: Url) -> Self {
        Client {
            inner: rpc::Client::new(base_url),
        }
    }

    /// Execute RPC method: `web3_clientVersion`. Return version string:
    /// "Geth/v1.10.2-unstable-f304290b-20210323/linux-amd64/go1.13.8"
    fn client_version(&self) -> Result<String> {
        let version = self
            .inner
            .send::<Vec<()>, String>(rpc::Request::v2("web3_clientVersion", vec![]))?;

        Ok(version)
    }

    /// Execute RPC method: `net_version`. Return network id (chain id).
    fn chain_id(&self) -> Result<ChainId> {
        let chain_id = self
            .inner
            .send::<Vec<()>, String>(rpc::Request::v2("net_version", vec![]))
            .context("failed to fetch net version")?;
        let chain_id: u32 = chain_id.parse()?;
        let chain_id = ChainId::from(chain_id);

        Ok(chain_id)
    }

    /// Execute RPC method: `eth_sendRawTransaction`. Return transaction hash.
    fn send_raw_transaction(&self, transaction_hex: String) -> Result<H256> {
        let tx_hash = self
            .inner
            .send(rpc::Request::v2("eth_sendRawTransaction", vec![
                transaction_hex,
            ]))
            .context("failed to send raw transaction")?;

        Ok(tx_hash)
    }

    /// Execute RPC method: `eth_getTransactionReceipt`.
    fn get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> Result<Option<TransactionReceipt>> {
        let receipt = self
            .inner
            .send(rpc::Request::v2("eth_getTransactionReceipt", vec![
                rpc::serialize(transaction_hash)?,
            ]))
            .context("failed to get transaction receipt")?;

        Ok(receipt)
    }

    /// Execute RPC method: `eth_getTransactionCount`. Return the number of
    /// transactions sent from this address.
    fn get_transaction_count(&self, account: Address, height: BlockNumber) -> Result<u32> {
        let count: String = self
            .inner
            .send(rpc::Request::v2("eth_getTransactionCount", vec![
                rpc::serialize(account)?,
                rpc::serialize(height)?,
            ]))
            .context("failed to get transaction count")?;

        let count = u32::from_str_radix(&count[2..], 16)?;
        Ok(count)
    }

    fn get_balance(&self, address: Address, height: BlockNumber) -> Result<Ether> {
        let amount: String = self
            .inner
            .send(rpc::Request::v2("eth_getBalance", vec![
                rpc::serialize(address)?,
                rpc::serialize(height)?,
            ]))
            .context("failed to get balance")?;
        let amount = Wei::try_from_hex_str(&amount)?;

        Ok(amount.into())
    }

    fn gas_price(&self) -> Result<Ether> {
        let amount = self
            .inner
            .send::<Vec<()>, String>(rpc::Request::v2("eth_gasPrice", vec![]))
            .context("failed to get gas price")?;
        let amount = Wei::try_from_hex_str(&amount[2..])?;

        Ok(amount.into())
    }

    fn gas_limit(&self, request: CallRequest, height: BlockNumber) -> Result<Uint256> {
        let gas_limit: String = self
            .inner
            .send(rpc::Request::v2("eth_estimateGas", vec![
                rpc::serialize(request)?,
                rpc::serialize(height)?,
            ]))
            .context("failed to get gas price")?;
        let gas_limit = Uint256::from_str_radix(&gas_limit[2..], 16)?;

        Ok(gas_limit)
    }
}
