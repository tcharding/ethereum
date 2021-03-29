#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::dbg_macro
)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![forbid(unsafe_code)]

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::num::ParseIntError;

use secp256k1::Secp256k1;
pub use secp256k1::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

pub use crate::asset::{Erc20, Ether, Gwei, Wei};
use crate::types::Address;

pub mod api;
pub mod asset;
pub mod geth;
pub mod jsonrpc_reqwest;
pub mod jsonrpc_ureq;
pub mod types;

/// Gets the address of a private key.
pub fn address_from_secret_key(sk: &SecretKey) -> Address {
    let secp = Secp256k1::signing_only();
    let pk = PublicKey::from_secret_key(&secp, sk);
    address_from_public_key(&pk)
}

/// Gets the address of a public key.
///
/// The public address is defined as the low 20 bytes of the keccak hash of
/// the public key. Note that the public key returned from the `secp256k1`
/// crate is 65 bytes long, that is because it is prefixed by `0x04` to
/// indicate an uncompressed public key; this first byte is ignored when
/// computing the hash.
pub fn address_from_public_key(pk: &PublicKey) -> Address {
    let pk = pk.serialize_uncompressed();

    debug_assert_eq!(pk[0], 0x04);
    let hash = keccak256(&pk[1..]);

    Address::from_slice(&hash[12..])
}

/// Compute the Keccak-256 hash of input bytes.
pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ChainId(u32);

/// Identifiers used by various networks in the Ethereum ecosystem.
impl ChainId {
    pub const MAINNET: Self = ChainId(1);
    pub const ROPSTEN: Self = ChainId(3);
    pub const KOVAN: Self = ChainId(42);
    pub const GETH_DEV: Self = ChainId(1337); // Arbitrary integer.
}

impl Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            &Self::MAINNET => write!(f, "MAINNET"),
            &Self::ROPSTEN => write!(f, "ROPSTEN"),
            &Self::KOVAN => write!(f, "KOVAN"),
            &Self::GETH_DEV => write!(f, "GETH-DEV"),
            other => write!(f, "UNKNOWN ({})", other.0),
        }
    }
}

impl From<ChainId> for u32 {
    fn from(chain_id: ChainId) -> Self {
        chain_id.0
    }
}

impl From<u32> for ChainId {
    fn from(id: u32) -> Self {
        ChainId(id)
    }
}

impl TryFrom<String> for ChainId {
    type Error = ParseIntError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let chain_id: u32 = s.parse()?;
        Ok(ChainId::from(chain_id))
    }
}
