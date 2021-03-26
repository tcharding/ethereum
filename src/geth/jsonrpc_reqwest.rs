//! JSON RPC client for go-ethereum, uses `reqwest` by way of
//! `../jsonrpc_reqwest`. ref: https://eth.wiki/json-rpc/API
use anyhow::{Context, Result};
use async_trait::async_trait;
use clarity::Uint256;

use crate::geth::{DefaultBlock, EthCall, GethClientAsync};
pub use crate::jsonrpc_reqwest::Url;
use crate::{Address, ChainId, Erc20, Ether, Gwei, Hash, TransactionReceipt, UnformattedData, Wei};

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
    async fn send_raw_transaction(&self, transaction_hex: String) -> Result<Hash> {
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
        transaction_hash: Hash,
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
    async fn get_transaction_count(&self, account: Address, height: DefaultBlock) -> Result<u32> {
        let count: String = self
            .inner
            .send(rpc::Request::v2("eth_getTransactionCount", vec![
                rpc::serialize(account)?,
                rpc::serialize(height.to_string())?,
            ]))
            .await
            .context("failed to get transaction count")?;

        let count = u32::from_str_radix(&count[2..], 16)?;
        Ok(count)
    }

    async fn get_balance(&self, address: Address, height: DefaultBlock) -> Result<Ether> {
        let amount: String = self
            .inner
            .send(rpc::Request::v2("eth_getBalance", vec![
                rpc::serialize(address)?,
                rpc::serialize(height.to_string())?,
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

    async fn erc20_balance(&self, account: Address, token_contract: Address) -> Result<Erc20> {
        #[derive(Debug, serde::Serialize)]
        struct CallRequest {
            to: Address,
            data: UnformattedData,
        }

        let call_request = CallRequest {
            to: token_contract,
            data: UnformattedData(balance_of_fn(account)?),
        };

        let amount: String = self
            .inner
            .send(rpc::Request::v2("eth_call", vec![
                rpc::serialize(call_request)?,
                rpc::serialize("latest")?,
            ]))
            .await
            .context("failed to get erc20 token balance")?;
        let amount = Wei::try_from_hex_str(&amount)?;

        Ok(Erc20 {
            token_contract,
            amount,
        })
    }

    async fn gas_limit(&self, request: EthCall, height: DefaultBlock) -> Result<Uint256> {
        let gas_limit: String = self
            .inner
            .send(rpc::Request::v2("eth_estimateGas", vec![
                rpc::serialize(request)?,
                rpc::serialize(height.to_string())?,
            ]))
            .await
            .context("failed to get gas price")?;
        let gas_limit = Uint256::from_str_radix(&gas_limit[2..], 16)?;

        Ok(gas_limit)
    }
}

fn balance_of_fn(account: Address) -> Result<Vec<u8>> {
    let account = clarity::Address::from_slice(account.as_bytes())
        .map_err(|_| anyhow::anyhow!("Could not construct clarity::Address from slice"))?;

    let balance_of =
        clarity::abi::encode_call("balanceOf(address)", &[clarity::abi::Token::Address(
            account,
        )])?;

    Ok(balance_of)
}
