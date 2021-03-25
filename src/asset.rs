use std::marker::Sized;

use conquer_once::Lazy;
use num::bigint::ParseBigIntError;
use num::BigUint;

pub use erc20::*;
pub use ether::*;
pub use wei::*; // TODO: Consider making this private.

mod erc20;
mod ether;
mod wei;

static WEI_IN_ETHER_U128: Lazy<u128> = Lazy::new(|| (10u128).pow(18));
static WEI_IN_ETHER_BIGUINT: Lazy<BigUint> = Lazy::new(|| BigUint::from(*WEI_IN_ETHER_U128));

pub trait FromWei<W> {
    fn from_wei(wei: W) -> Self;
}

pub trait TryFromWei<W>
where
    Self: Sized,
{
    fn try_from_wei(wei: W) -> Result<Self, Error>;
}

#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("value provided overflows")]
    Overflow,
    #[error("parsing error encountered")]
    Parse(#[from] ParseBigIntError),
}
