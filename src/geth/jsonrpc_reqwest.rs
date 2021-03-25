//! JSON RPC client for go-ethereum, uses `reqwest` by way of
//! `../jsonrpc_reqwest`. ref: https://eth.wiki/json-rpc/API
use std::str::FromStr;

use anyhow::{Context, Result};
use clarity::Uint256;
use ethereum_types::U256;
pub use url::Url;

use crate::{
    jsonrpc_reqwest, Address, Amount, ChainId, Erc20, Ether, Hash, TransactionReceipt,
    UnformattedData,
};

#[derive(Debug, Clone)]
pub struct Client {
    inner: jsonrpc_reqwest::Client,
}

impl Client {
    pub fn new(url: Url) -> Self {
        Client {
            inner: jsonrpc_reqwest::Client::new(url),
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
            .send::<Vec<()>, String>(jsonrpc_reqwest::Request::v2("web3_clientVersion", vec![]))
            .await
            .context("failed to fetch client version")?;

        Ok(version)
    }

    /// Execute RPC method: `net_version`. Return network id (chain id).
    pub async fn chain_id(&self) -> Result<ChainId> {
        let chain_id = self
            .inner
            .send::<Vec<()>, String>(jsonrpc_reqwest::Request::v2("net_version", vec![]))
            .await
            .context("failed to fetch net version")?;
        let chain_id: u32 = chain_id.parse()?;
        let chain_id = ChainId::from(chain_id);

        Ok(chain_id)
    }

    /// Execute RPC method: `eth_sendRawTransaction`. Return transaction hash.
    pub async fn send_raw_transaction(&self, transaction_hex: String) -> Result<Hash> {
        let tx_hash = self
            .inner
            .send(jsonrpc_reqwest::Request::v2(
                "eth_sendRawTransaction",
                vec![transaction_hex],
            ))
            .await
            .context("failed to send raw transaction")?;

        Ok(tx_hash)
    }

    /// Execute RPC method: `eth_getTransactionReceipt`.
    pub async fn get_transaction_receipt(
        &self,
        transaction_hash: Hash,
    ) -> Result<Option<TransactionReceipt>> {
        let receipt = self
            .inner
            .send(jsonrpc_reqwest::Request::v2(
                "eth_getTransactionReceipt",
                vec![jsonrpc_reqwest::serialize(transaction_hash)?],
            ))
            .await
            .context("failed to get transaction receipt")?;

        Ok(receipt)
    }

    /// Execute RPC method: `eth_getTransactionCount`. Return the number of
    /// transactions sent from this address.
    pub async fn get_transaction_count(&self, account: Address) -> Result<u32> {
        let count: String = self
            .inner
            .send(jsonrpc_reqwest::Request::v2(
                "eth_getTransactionCount",
                vec![
                    jsonrpc_reqwest::serialize(account)?,
                    jsonrpc_reqwest::serialize("latest")?,
                ],
            ))
            .await
            .context("failed to get transaction count")?;

        let count = u32::from_str_radix(&count[2..], 16)?;
        Ok(count)
    }

    pub async fn erc20_balance(&self, account: Address, token_contract: Address) -> Result<Erc20> {
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
            .send(jsonrpc_reqwest::Request::v2("eth_call", vec![
                jsonrpc_reqwest::serialize(call_request)?,
                jsonrpc_reqwest::serialize("latest")?,
            ]))
            .await
            .context("failed to get erc20 token balance")?;
        let amount = Amount::try_from_hex_str(&amount)?;

        Ok(Erc20 {
            token_contract,
            amount,
        })
    }

    pub async fn get_balance(&self, address: Address) -> Result<Ether> {
        let amount: String = self
            .inner
            .send(jsonrpc_reqwest::Request::v2("eth_getBalance", vec![
                jsonrpc_reqwest::serialize(address)?,
                jsonrpc_reqwest::serialize("latest")?,
            ]))
            .await
            .context("failed to get balance")?;
        let amount = Ether::try_from_hex_str(&amount)?;

        Ok(amount)
    }

    pub async fn gas_price(&self) -> Result<Ether> {
        let amount = self
            .inner
            .send::<Vec<()>, String>(jsonrpc_reqwest::Request::v2("eth_gasPrice", vec![]))
            .await
            .context("failed to get gas price")?;
        let amount = Ether::try_from_hex_str(&amount[2..])?;

        Ok(amount)
    }

    pub async fn gas_limit(&self, request: EstimateGasRequest) -> Result<clarity::Uint256> {
        let gas_limit: String = self
            .inner
            .send(jsonrpc_reqwest::Request::v2("eth_estimateGas", vec![
                jsonrpc_reqwest::serialize(request)?,
            ]))
            .await
            .context("failed to get gas price")?;
        let gas_limit = clarity::Uint256::from_str_radix(&gas_limit[2..], 16)?;

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
