//! JSON RPC clients for go-ethereum.
//! ref: https://eth.wiki/json-rpc/API
use std::fmt::{self, Debug, Display, Formatter};

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
