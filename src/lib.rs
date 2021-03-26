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
use std::fmt::{self, Display};
use std::num::ParseIntError;

use secp256k1::Secp256k1;
pub use secp256k1::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

pub use crate::asset::{Erc20, Ether, Gwei, Wei};
pub use crate::types::Address;

pub mod asset;
pub mod geth;
pub mod jsonrpc_reqwest;
pub mod jsonrpc_ureq;
pub mod types;

/// Gets the public address of a private key.
pub fn secret_key_address(key: &SecretKey) -> Address {
    let secp = Secp256k1::signing_only();
    let public_key = PublicKey::from_secret_key(&secp, key);
    public_key_address(&public_key)
}

/// Gets the address of a public key.
///
/// The public address is defined as the low 20 bytes of the keccak hash of
/// the public key. Note that the public key returned from the `secp256k1`
/// crate is 65 bytes long, that is because it is prefixed by `0x04` to
/// indicate an uncompressed public key; this first byte is ignored when
/// computing the hash.
pub fn public_key_address(public_key: &PublicKey) -> Address {
    let public_key = public_key.serialize_uncompressed();

    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

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

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// A serde module for formatting bytes according to Ethereum's convention for
/// "data".
///
/// See https://eth.wiki/json-rpc/API#hex-value-encoding for more details.
pub mod serde_hex_data {
    use super::*;
    use hex::FromHex;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S, V>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: AsRef<[u8]>,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(value.as_ref())))
    }

    pub fn deserialize<'de, D, V>(deserializer: D) -> Result<V, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
        V: FromHex,
        <V as FromHex>::Error: Display,
    {
        let string = String::deserialize(deserializer)?;
        let value = V::from_hex(string.trim_start_matches("0x")).map_err(D::Error::custom)?;

        Ok(value)
    }
}
