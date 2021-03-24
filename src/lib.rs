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
use std::str::FromStr;

use anyhow::Result;
pub use ethbloom::Bloom as H2048;
use hex::FromHexError;
pub use primitive_types::U256;
use serde::{Deserialize, Deserializer, Serialize};

pub use crate::asset::{Amount, Erc20, Ether};

pub mod asset;
pub mod geth;

/// Ethereum address size is 20 bytes (the last 20 bytes of the Keccak hashed
/// pubkey).
pub const ADDR_SIZE: usize = 20;

#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Address(#[serde(with = "serde_hex_data")] [u8; ADDR_SIZE]);

impl Address {
    pub fn as_bytes(&self) -> &[u8; ADDR_SIZE] {
        &self.0
    }

    /// Generates a random address for use in tests.
    #[cfg(test)]
    pub fn random() -> Address {
        use rand::RngCore;

        let mut buffer = [0u8; ADDR_SIZE];
        rand::thread_rng().fill_bytes(&mut buffer);

        Address(buffer)
    }
}

impl From<[u8; ADDR_SIZE]> for Address {
    fn from(bytes: [u8; ADDR_SIZE]) -> Self {
        Address(bytes)
    }
}

impl From<Address> for [u8; ADDR_SIZE] {
    fn from(s: Address) -> Self {
        s.0
    }
}

impl FromStr for Address {
    type Err = FromHexError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        let mut address = [0u8; ADDR_SIZE];
        hex::decode_to_slice(hex.trim_start_matches("0x"), &mut address)?;

        Ok(Address(address))
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0 {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl From<Address> for Hash {
    fn from(address: Address) -> Self {
        let mut h256 = Hash([0u8; 32]);
        h256.0[(32 - ADDR_SIZE)..32].copy_from_slice(&address.0);
        h256
    }
}

#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Hash(#[serde(with = "serde_hex_data")] [u8; 32]);

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(s: Hash) -> Self {
        s.0
    }
}

impl Hash {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0 {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl FromStr for Hash {
    type Err = FromHexError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        let mut hash = [0u8; 32];
        hex::decode_to_slice(hex.trim_start_matches("0x"), &mut hash)?;

        Ok(Hash(hash))
    }
}

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct TransactionReceipt {
    /// Contract address created, or `None` if not a deployment.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<Address>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// Status: Whether or not the transaction executed successfully
    #[serde(rename = "status", deserialize_with = "deserialize_status")]
    pub successful: bool,
    /// The block number this transaction was included in.
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U256>,
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<bool, <D as Deserializer<'de>>::Error>
where
    D: Deserializer<'de>,
{
    let hex_string = String::deserialize(deserializer)?;
    Ok(&hex_string == "0x1")
}

/// Description of a Transaction, pending or in the chain.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Transaction {
    /// Transaction hash.
    pub hash: Hash,
    /// Recipient (None when contract creation).
    pub to: Option<Address>,
    /// Transfered value.
    pub value: U256,
    /// Input data.
    pub input: UnformattedData,
}

/// A log produced by a transaction.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Log {
    /// Address (H160)
    pub address: Address,
    /// List of topics.
    pub topics: Vec<Hash>,
    /// Data.
    pub data: UnformattedData,
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Hash,
}

/// The block returned from RPC calls.
///
/// This type contains only the fields we are actually using.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Block {
    /// Hash of the block.
    pub hash: Hash,
    /// Hash of the parent block.
    #[serde(rename = "parentHash")]
    pub parent_hash: Hash,
    /// Bloom filter logs.
    #[serde(rename = "logsBloom")]
    pub logs_bloom: H2048,
    /// Block timestamp .
    pub timestamp: U256,
    /// List of transactions in the block.
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnformattedData(#[serde(with = "serde_hex_data")] pub Vec<u8>);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialise_address() {
        let json =
            serde_json::Value::String("0xc5549e335b2786520f4c5d706c76c9ee69d0a028".to_owned());
        let _: Address = Address::deserialize(&json).unwrap();
    }

    #[test]
    fn from_string_address() {
        let json =
            serde_json::Value::String("0xc5549e335b2786520f4c5d706c76c9ee69d0a028".to_owned());
        let deserialized: Address = Address::deserialize(&json).unwrap();

        let from_string = Address::from_str("0xc5549e335b2786520f4c5d706c76c9ee69d0a028").unwrap();

        assert_eq!(from_string, deserialized);
    }

    #[test]
    fn deserialise_hash() {
        let json = serde_json::Value::String(
            "0x3ae3b6ffb04204f52dee42000e8b971c0f7c2b4aa8dd9455e41a30ee4b31e8a9".to_owned(),
        );
        let _: Hash = Hash::deserialize(&json).unwrap();
    }

    #[test]
    fn deserialise_hash_when_not_using_reference_to_deserialize_fails() {
        // This is due to a bug in serde-jex, keep this test until https://github.com/fspmarshall/serde-hex/pull/8
        // is fixed.
        let json = serde_json::Value::String(
            "0x3ae3b6ffb04204f52dee42000e8b971c0f7c2b4aa8dd9455e41a30ee4b31e8a9".to_owned(),
        );

        let deserialized = serde_json::from_value::<Hash>(json);
        matches!(deserialized, Err(_));
    }

    #[test]
    fn deserialise_log() {
        let json = r#"
            {
                "address": "0xc5549e335b2786520f4c5d706c76c9ee69d0a028",
                "blockHash": "0x3ae3b6ffb04204f52dee42000e8b971c0f7c2b4aa8dd9455e41a30ee4b31e8a9",
                "blockNumber": "0x856ca0",
                "data": "0x0000000000000000000000000000000000000000000000000000000ba43b7400",
                "logIndex": "0x81",
                "removed": false,
                "topics": [
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                    "0x000000000000000000000000fb303a1fba5b4804863131145bc27256d3ab6692",
                    "0x000000000000000000000000d50fb7d948426633ec126aeea140ce4dd0979682"
                ],
                "transactionHash": "0x5ffd218c617f76c73aa49ee636027440b58eb022778c5e75794563c0d60fcb88",
                "transactionIndex": "0x93"
            }"#;

        let _: Log = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn deserialize_receipt_with_status_1() {
        let json = r#"
        {
          "contractAddress": null,
          "logs": [],
          "status": "0x1"
        }
        "#;

        let receipt = serde_json::from_str::<TransactionReceipt>(json).unwrap();

        assert_eq!(receipt.successful, true);
    }

    #[test]
    fn deserialize_receipt_with_status_0() {
        let json = r#"
        {
          "contractAddress": null,
          "logs": [],
          "status": "0x0"
        }
        "#;

        let receipt = serde_json::from_str::<TransactionReceipt>(json).unwrap();

        assert_eq!(receipt.successful, false);
    }
}
