//! JSON RPC clients for go-ethereum. Client modules are named after the library
//! they rely on. ref: https://eth.wiki/json-rpc/API
use anyhow::Result;
use async_trait::async_trait;
use clarity::Uint256;

use crate::jsonrpc_ureq::Url;
use crate::types::{BlockNumber, CallRequest, TransactionReceipt, H256};
use crate::{Address, ChainId, Ether, Gwei};

pub mod jsonrpc_client; // Uses the `jsonrpc_client` library.
pub mod jsonrpc_reqwest; // Uses the `reqwest` library.
pub mod jsonrpc_ureq; // Uses the `ureq` library.

/// A go-ethereum client.
// If you edit this please edit `GethClientAsync` as well.
pub trait GethClient {
    fn new(base_url: Url) -> Self;

    /// Execute RPC method: `web3_clientVersion`. Return version string:
    /// "Geth/v1.10.2-unstable-f304290b-20210323/linux-amd64/go1.13.8"
    fn client_version(&self) -> Result<String>;

    /// Execute RPC method: `net_version`. Return network id (chain id).
    fn chain_id(&self) -> Result<ChainId>;

    /// Execute RPC method: `eth_sendRawTransaction`. Return transaction hash.
    fn send_raw_transaction(&self, transaction_hex: String) -> Result<H256>;

    /// Execute RPC method: `eth_getTransactionReceipt`.
    fn get_transaction_receipt(&self, transaction_hash: H256)
        -> Result<Option<TransactionReceipt>>;

    /// Execute RPC method: `eth_getTransactionCount`. Return the number of
    /// transactions sent from this address.
    fn get_transaction_count(&self, account: Address, height: BlockNumber) -> Result<u32>;

    fn get_balance(&self, address: Address, height: BlockNumber) -> Result<Ether>;

    fn gas_price(&self) -> Result<Ether>;

    fn gas_limit(&self, request: CallRequest, height: BlockNumber) -> Result<Uint256>;
}

/// This is exactly the same as `GethClient` except with `async` methods.
#[async_trait]
pub trait GethClientAsync {
    fn new(base_url: Url) -> Self;

    /// Execute RPC method: `web3_clientVersion`. Return version string:
    /// "Geth/v1.10.2-unstable-f304290b-20210323/linux-amd64/go1.13.8"
    async fn client_version(&self) -> Result<String>;

    /// Execute RPC method: `net_version`. Return network id (chain id).
    async fn chain_id(&self) -> Result<ChainId>;

    /// Execute RPC method: `eth_sendRawTransaction`. Return transaction hash.
    async fn send_raw_transaction(&self, transaction_hex: String) -> Result<H256>;

    /// Execute RPC method: `eth_getTransactionReceipt`.
    async fn get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> Result<Option<TransactionReceipt>>;

    /// Execute RPC method: `eth_getTransactionCount`. Return the number of
    /// transactions sent from this address.
    async fn get_transaction_count(&self, account: Address, height: BlockNumber) -> Result<u32>;

    async fn get_balance(&self, address: Address, height: BlockNumber) -> Result<Ether>;

    async fn gas_price(&self) -> Result<Gwei>;

    async fn gas_limit(&self, request: CallRequest, height: BlockNumber) -> Result<Uint256>;
}
