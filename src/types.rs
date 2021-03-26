//! Etherum types copied from `rust-web3`
//! ref: https://github.com/tomusdrw/rust-web3

mod bytes;
mod transaction_request;

pub use self::{
    bytes::Bytes,
    transaction_request::{CallRequest, TransactionCondition, TransactionRequest},
};
