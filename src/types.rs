//! Etherum types copied from `rust-web3`
//! ref: https://github.com/tomusdrw/rust-web3

mod bytes;
mod transaction_request;
mod uint;

pub use self::{
    bytes::Bytes,
    transaction_request::{CallRequest, TransactionCondition, TransactionRequest},
    uint::{H128, H160, H2048, H256, H512, H520, H64, U128, U256, U64},
};

/// Address
// FIXME: Should we use this?
// pub type Address = H160;
/// Index in block
pub type Index = U64;
