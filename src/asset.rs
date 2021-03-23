use std::{fmt, str::FromStr};

use conquer_once::Lazy;
use num::bigint::ParseBigIntError;
use num::pow::Pow;
use num::{BigUint, Integer, Num, Zero};
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::{Address, U256};

static WEI_IN_ETHER_U128: Lazy<u128> = Lazy::new(|| (10u128).pow(18));
static WEI_IN_ETHER_BIGUINT: Lazy<BigUint> = Lazy::new(|| BigUint::from(*WEI_IN_ETHER_U128));

/// Ether, the native token of the Ethereum chain.
pub type Ether = Amount;

/// ERC-20 standard token.
#[derive(Debug, Deserialize, Serialize, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Erc20 {
    pub token_contract: Address,
    pub amount: Amount,
}

impl fmt::Display for Erc20 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.amount)
    }
}

impl Erc20 {
    pub fn new(token_contract: Address, amount: Amount) -> Self {
        Erc20 {
            token_contract,
            amount,
        }
    }
}

pub trait FromWei<W> {
    fn from_wei(wei: W) -> Self;
}

pub trait TryFromWei<W>
where
    Self: std::marker::Sized,
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

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Amount(BigUint);

impl Amount {
    pub fn zero() -> Self {
        Self(BigUint::zero())
    }

    pub fn max_value() -> Self {
        Self(BigUint::from(2u8).pow(256u32) - 1u8)
    }

    pub fn to_wei_dec(&self) -> String {
        self.0.to_str_radix(10)
    }

    // TODO: Rename this try_from_...
    pub fn from_wei_dec_str(str: &str) -> Result<Self, Error> {
        let int = BigUint::from_str_radix(str, 10)?;
        Self::try_from_wei(int)
    }

    pub fn to_u256(&self) -> U256 {
        let buf = self.0.to_bytes_be();
        U256::from_big_endian(&buf)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes_le()
    }

    pub fn try_from_hex_str(hex: &str) -> Result<Self, Error> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        let int = BigUint::from_str_radix(hex, 16)?;
        let ether = Amount::try_from_wei(int)?;

        Ok(ether)
    }

    pub fn checked_mul(self, factor: u64) -> Option<Self> {
        let result = Self(self.0 * factor);

        if result > Self::max_value() {
            return None;
        }

        Some(result)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let (ether, rem) = self.0.div_rem(&WEI_IN_ETHER_BIGUINT);

        if rem.is_zero() {
            write!(f, "{} ETH", ether)
        } else {
            // format number as base 10
            let rem = rem.to_str_radix(10);

            // prefix with 0 in the front until we have 18 chars
            let rem = format!("{:0>18}", rem);

            // trim unnecessary 0s from the back
            let rem = rem.trim_end_matches('0');

            write!(f, "{}.{} ETH", ether, rem)
        }
    }
}

macro_rules! impl_from_wei_primitive {
    ($primitive:ty) => {
        impl FromWei<$primitive> for Amount {
            fn from_wei(w: $primitive) -> Self {
                Amount(BigUint::from(w))
            }
        }
    };
}

impl_from_wei_primitive!(u8);
impl_from_wei_primitive!(u16);
impl_from_wei_primitive!(u32);
impl_from_wei_primitive!(u64);
impl_from_wei_primitive!(u128);

impl FromWei<U256> for Amount {
    fn from_wei(wei: U256) -> Self {
        let mut buf = [0u8; 32];
        wei.to_big_endian(&mut buf);
        Amount(BigUint::from_bytes_be(&buf))
    }
}

impl TryFromWei<BigUint> for Amount {
    fn try_from_wei(wei: BigUint) -> Result<Self, Error> {
        if wei > Self::max_value().0 {
            Err(Error::Overflow)
        } else {
            Ok(Self(wei))
        }
    }
}

impl TryFromWei<&str> for Amount {
    fn try_from_wei(string: &str) -> Result<Amount, Error> {
        let uint = BigUint::from_str(string)?;
        Ok(Self(uint))
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'vde> de::Visitor<'vde> for Visitor {
            type Value = Amount;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                formatter.write_str("A string representing a wei quantity")
            }

            fn visit_str<E>(self, v: &str) -> Result<Amount, E>
            where
                E: de::Error,
            {
                let wei = BigUint::from_str(v).map_err(E::custom)?;
                let quantity = Amount::try_from_wei(wei).map_err(E::custom)?;
                Ok(quantity)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_one_thousand_in_u256_equals_one_thousand_u32() {
        let u256 = U256::from(1_000);
        let u256 = Amount::from_wei(u256);
        let u32 = Amount::from_wei(1_000u32);

        assert_eq!(u256, u32)
    }

    #[test]
    fn from_one_thousand_in_u32_converts_to_u256() {
        let ether = Amount::from_wei(1_000u32);
        let u256 = U256::from(1_000);

        assert_eq!(ether.to_u256(), u256)
    }

    #[test]
    fn given_9000_exa_wei_display_in_ether() {
        assert_eq!(
            Amount::from_wei(9_000 * *WEI_IN_ETHER_U128).to_string(),
            "9000 ETH"
        );
    }

    #[test]
    fn given_1_peta_wei_display_in_ether() {
        assert_eq!(
            Amount::from_wei(1_000_000_000_000_000u128).to_string(),
            "0.001 ETH"
        );
    }

    #[test]
    fn given_some_weird_wei_number_formats_correctly_as_eth() {
        assert_eq!(
            Amount::from_wei(1_003_564_412_000_000_000u128).to_string(),
            "1.003564412 ETH"
        );
    }

    #[test]
    fn try_from_wei_dec_str_equals_from_wei_u128() {
        let from_str = Amount::try_from_wei("9001000000000000000000").unwrap();
        let from_u128 = Amount::from_wei(9_001_000_000_000_000_000_000u128);

        assert_eq!(from_str, from_u128)
    }

    #[test]
    fn serialize() {
        let ether = Amount::from_wei(*WEI_IN_ETHER_U128);
        let ether_str = serde_json::to_string(&ether).unwrap();
        assert_eq!(ether_str, "\"1000000000000000000\"");
    }

    #[test]
    fn deserialize() {
        let ether_str = "\"1000000000000000000\"";
        let ether = serde_json::from_str::<Amount>(ether_str).unwrap();
        assert_eq!(ether, Amount::from_wei(*WEI_IN_ETHER_U128));
    }

    #[test]
    fn given_too_big_biguint_return_overflow_error() {
        let wei = BigUint::from_slice(&[
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX, // 9th u32, should make it over u256
        ]);
        let quantity = Amount::try_from_wei(wei);
        assert_eq!(quantity, Err(Error::Overflow))
    }

    #[test]
    fn given_max_u256_it_does_not_overflow() {
        let wei = BigUint::from_slice(&[
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
            std::u32::MAX,
        ]);
        let quantity = Amount::try_from_wei(wei);
        assert!(quantity.is_ok())
    }

    #[test]
    fn given_too_big_string_when_deserializing_return_overflow_error() {
        let quantity_str =
            "\"115792089237316195423570985008687907853269984665640564039457584007913129639936\""; // This is Amount::max_value() + 1
        let res = serde_json::from_str::<Amount>(quantity_str);
        assert!(res.is_err())
    }

    #[test]
    fn to_dec() {
        let ether = Amount::from_wei(12_345u32);
        assert_eq!(ether.to_wei_dec(), "12345".to_string())
    }

    #[test]
    fn given_str_of_wei_in_dec_format_instantiate_ether() {
        let ether = Amount::from_wei_dec_str("12345").unwrap();
        assert_eq!(ether, Amount::from_wei(12_345u32))
    }

    #[test]
    fn given_str_above_u256_max_in_dec_format_return_overflow() {
        let res = Amount::from_wei_dec_str(
            "115792089237316195423570985008687907853269984665640564039457584007913129639936",
        ); // This is Amount::max_value() + 1
        assert_eq!(res, Err(Error::Overflow))
    }

    #[test]
    fn display() {
        let quantity = Amount::from_wei(123_456_789u64);
        assert_eq!(quantity.to_string(), "123456789".to_string());
    }
}
