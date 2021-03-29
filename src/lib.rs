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

pub use clarity::Address;
use secp256k1::Secp256k1;
pub use secp256k1::{PublicKey, SecretKey};

pub mod api;
pub mod jsonrpc;
pub mod types;

/// Gets the address of a private key.
pub fn address_from_secret_key(sk: &SecretKey) -> Result<Address, clarity::Error> {
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
pub fn address_from_public_key(pk: &PublicKey) -> Result<Address, clarity::Error> {
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
