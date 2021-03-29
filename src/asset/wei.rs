use std::convert::TryFrom;
use std::ops::{Add, Sub};
use std::{fmt, str::FromStr};

use clarity::Uint256;
use num::pow::Pow;
use num::{BigUint, Integer, Num, Zero};
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::asset::{Error, WEI_IN_ETHER_BIGUINT};
use crate::types::U256;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Wei(BigUint);

impl Wei {
    pub fn zero() -> Self {
        Self(BigUint::zero())
    }

    pub fn max_value() -> Self {
        Self(BigUint::from(2u8).pow(256u32) - 1u8)
    }

    pub fn to_dec_string(&self) -> String {
        self.0.to_str_radix(10)
    }

    pub fn try_from_dec_str(str: &str) -> Result<Self, Error> {
        let int = BigUint::from_str_radix(str, 10)?;
        Self::try_from(int)
    }

    pub fn to_hex_string(&self) -> String {
        todo!()
    }

    pub fn try_from_hex_str(hex: &str) -> Result<Self, Error> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        let int = BigUint::from_str_radix(hex, 16)?;
        let ether = Wei::try_from(int)?;

        Ok(ether)
    }

    pub fn to_u256(&self) -> U256 {
        let buf = self.0.to_bytes_be();
        U256::from_big_endian(&buf)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes_le()
    }

    pub fn checked_mul(self, factor: u64) -> Option<Self> {
        let result = Self(self.0 * factor);

        if result > Self::max_value() {
            return None;
        }

        Some(result)
    }

    pub fn div_by_wei(&self) -> (BigUint, BigUint) {
        self.0.div_rem(&WEI_IN_ETHER_BIGUINT)
    }
}

impl fmt::Display for Wei {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // TODO: Implement Display for Wei.
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_from_primitive {
    ($primitive:ty) => {
        impl From<$primitive> for Wei {
            fn from(w: $primitive) -> Self {
                Wei(BigUint::from(w))
            }
        }
    };
}

impl_from_primitive!(u8);
impl_from_primitive!(u16);
impl_from_primitive!(u32);
impl_from_primitive!(u64);
impl_from_primitive!(u128);

impl From<Uint256> for Wei {
    fn from(wei: Uint256) -> Self {
        Self((*wei).clone())
    }
}

impl From<Wei> for Uint256 {
    fn from(wei: Wei) -> Self {
        Uint256(wei.0)
    }
}

impl From<U256> for Wei {
    fn from(wei: U256) -> Self {
        let mut buf = [0u8; 32];
        wei.to_big_endian(&mut buf);
        Wei(BigUint::from_bytes_be(&buf))
    }
}

impl TryFrom<BigUint> for Wei {
    type Error = Error;

    fn try_from(wei: BigUint) -> Result<Self, Self::Error> {
        if wei > Self::max_value().0 {
            Err(Error::Overflow)
        } else {
            Ok(Self(wei))
        }
    }
}

impl TryFrom<&str> for Wei {
    type Error = Error;

    fn try_from(s: &str) -> Result<Wei, Self::Error> {
        Wei::try_from_dec_str(s)
    }
}

impl<'de> Deserialize<'de> for Wei {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'vde> de::Visitor<'vde> for Visitor {
            type Value = Wei;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                formatter.write_str("A string representing a wei amount")
            }

            fn visit_str<E>(self, v: &str) -> Result<Wei, E>
            where
                E: de::Error,
            {
                let wei = BigUint::from_str(v).map_err(E::custom)?;
                let amount = Wei::try_from(wei).map_err(E::custom)?;
                Ok(amount)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for Wei {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl Add for Wei {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.add(other.0))
    }
}

impl Sub for Wei {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0.sub(other.0))
    }
}

#[derive(Clone, Debug)]
pub struct Gwei(Wei);

impl From<Wei> for Gwei {
    fn from(wei: Wei) -> Self {
        Self(wei)
    }
}

impl fmt::Display for Gwei {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "TODO: implement Display for Gwei")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::asset::WEI_IN_ETHER_U128;

    use super::*;

    #[test]
    fn from_one_thousand_in_u256_equals_one_thousand_u32() {
        let u256 = U256::from(1_000);
        let u256 = Wei::from(u256);
        let u32 = Wei::from(1_000u32);

        assert_eq!(u256, u32)
    }

    #[test]
    fn from_one_thousand_in_u32_converts_to_u256() {
        let ether = Wei::from(1_000u32);
        let u256 = U256::from(1_000);

        assert_eq!(ether.to_u256(), u256)
    }

    #[test]
    fn try_from_dec_str_equals_from_u128() {
        let from_str = Wei::try_from("9001000000000000000000").unwrap();
        let from_u128 = Wei::from(9_001_000_000_000_000_000_000u128);

        assert_eq!(from_str, from_u128)
    }

    #[test]
    fn serialize() {
        let ether = Wei::from(*WEI_IN_ETHER_U128);
        let ether_str = serde_json::to_string(&ether).unwrap();
        assert_eq!(ether_str, "\"1000000000000000000\"");
    }

    #[test]
    fn deserialize() {
        let ether_str = "\"1000000000000000000\"";
        let ether = serde_json::from_str::<Wei>(ether_str).unwrap();
        assert_eq!(ether, Wei::from(*WEI_IN_ETHER_U128));
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
        let amount = Wei::try_from(wei);
        assert_eq!(amount, Err(Error::Overflow))
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
        let amount = Wei::try_from(wei);
        assert!(amount.is_ok())
    }

    #[test]
    fn given_too_big_string_when_deserializing_return_overflow_error() {
        let amount =
            "\"115792089237316195423570985008687907853269984665640564039457584007913129639936\""; // This is Wei::max_value() + 1
        let res = serde_json::from_str::<Wei>(amount);
        assert!(res.is_err())
    }

    #[test]
    fn to_decimal_string() {
        let wei = Wei::from(12_345u32);
        assert_eq!(wei.to_dec_string(), "12345".to_string())
    }

    #[test]
    fn from_decimal_string() -> Result<()> {
        let wei = Wei::try_from_dec_str("12345")?;
        assert_eq!(wei, Wei::from(12_345u32));
        Ok(())
    }

    #[test]
    fn given_str_above_u256_max_in_dec_format_return_overflow() -> Result<()> {
        let res = Wei::try_from_dec_str(
            "115792089237316195423570985008687907853269984665640564039457584007913129639936",
        ); // This is Wei::max_value() + 1
        assert_eq!(res, Err(Error::Overflow));
        Ok(())
    }
}
