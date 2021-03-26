//! JSON RPC clients for go-ethereum. Client modules are named after the library
//! they rely on. ref: https://eth.wiki/json-rpc/API
use std::fmt::{self, Debug, Display, Formatter};

use anyhow::Result;
use async_trait::async_trait;
use clarity::Uint256;

use crate::jsonrpc_ureq::Url;
use crate::transaction_request::CallRequest;
use crate::{Address, ChainId, Erc20, Ether, Gwei, Hash, TransactionReceipt};

pub mod jsonrpc_client; // Uses the `jsonrpc_client` library.
pub mod jsonrpc_reqwest; // Uses the `reqwest` library.
pub mod jsonrpc_ureq; // Uses the `ureq` library.

/// The default block parameter (see API ref at top of file).
#[derive(Clone, Copy, Debug)]
pub enum DefaultBlock {
    Num(u32),
    Earliest,
    Latest,
    Pending,
}

impl Display for DefaultBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DefaultBlock::Num(n) => write!(f, "0x{:x?}", n),
            DefaultBlock::Earliest => write!(f, "earliest"),
            DefaultBlock::Latest => write!(f, "latest"),
            DefaultBlock::Pending => write!(f, "pending"),
        }
    }
}

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
    fn send_raw_transaction(&self, transaction_hex: String) -> Result<Hash>;

    /// Execute RPC method: `eth_getTransactionReceipt`.
    fn get_transaction_receipt(&self, transaction_hash: Hash)
        -> Result<Option<TransactionReceipt>>;

    /// Execute RPC method: `eth_getTransactionCount`. Return the number of
    /// transactions sent from this address.
    fn get_transaction_count(&self, account: Address, height: DefaultBlock) -> Result<u32>;

    fn erc20_balance(&self, account: Address, token_contract: Address) -> Result<Erc20>;

    fn get_balance(&self, address: Address, height: DefaultBlock) -> Result<Ether>;

    fn gas_price(&self) -> Result<Ether>;

    fn gas_limit(&self, request: CallRequest, height: DefaultBlock) -> Result<Uint256>;
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
    async fn send_raw_transaction(&self, transaction_hex: String) -> Result<Hash>;

    /// Execute RPC method: `eth_getTransactionReceipt`.
    async fn get_transaction_receipt(
        &self,
        transaction_hash: Hash,
    ) -> Result<Option<TransactionReceipt>>;

    /// Execute RPC method: `eth_getTransactionCount`. Return the number of
    /// transactions sent from this address.
    async fn get_transaction_count(&self, account: Address, height: DefaultBlock) -> Result<u32>;

    async fn get_balance(&self, address: Address, height: DefaultBlock) -> Result<Ether>;

    async fn erc20_balance(&self, account: Address, token_contract: Address) -> Result<Erc20>;

    async fn gas_price(&self) -> Result<Gwei>;

    async fn gas_limit(&self, request: CallRequest, height: DefaultBlock) -> Result<Uint256>;
}
