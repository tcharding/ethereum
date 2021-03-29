use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub};

use clarity::Uint256;

use crate::asset::{Error, Wei, Zero};

/// Ether, the native token of the Ethereum chain.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Ether(Wei);

impl Ether {
    pub fn zero() -> Self {
        Self(Wei::zero())
    }

    pub fn to_dec_string(&self) -> String {
        let (ether, rem) = self.0.div_by_wei();

        if rem.is_zero() {
            format!("{}", ether)
        } else {
            // format number as base 10
            let rem = rem.to_str_radix(10);

            // prefix with 0 in the front until we have 18 chars
            let rem = format!("{:0>18}", rem);

            // trim unnecessary 0s from the back
            let rem = rem.trim_end_matches('0');

            format!("{}.{}", ether, rem)
        }
    }

    // This function is non-trivial to implement because floating point math is
    // tricky to write and error prone.
    pub fn try_from_dec_str(_: &str) -> Result<Self, Error> {
        todo!()
    }
}

impl Display for Ether {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} ETH", self.to_dec_string())
    }
}

impl From<Wei> for Ether {
    fn from(wei: Wei) -> Self {
        Self(wei)
    }
}

impl From<Uint256> for Ether {
    fn from(wei: Uint256) -> Self {
        Self(Wei::from(wei))
    }
}

impl From<Ether> for Uint256 {
    fn from(eth: Ether) -> Self {
        Uint256::from(eth.0)
    }
}

impl Add for Ether {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.add(other.0))
    }
}

impl Sub for Ether {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0.sub(other.0))
    }
}

#[cfg(test)]
mod tests {
    use crate::asset::WEI_IN_ETHER_U128;

    use super::*;

    #[test]
    fn display() {
        let amount = Ether::from(Wei::from(123_456_789u64));
        assert_eq!(amount.to_string(), "0.000000000123456789 ETH".to_string());
    }

    #[test]
    fn given_1_peta_wei_display() {
        let ether = Ether::from(Wei::from(1_000_000_000_000_000u128));
        assert_eq!(ether.to_string(), "0.001 ETH");
    }

    #[test]
    fn given_some_weird_wei_number_formats_correctly() {
        assert_eq!(
            Ether(Wei::from(1_003_564_412_000_000_000u128)).to_string(),
            "1.003564412 ETH"
        );
    }

    #[test]
    fn given_9000_exa_wei_display() {
        assert_eq!(
            Ether::from(Wei::from(9_000 * *WEI_IN_ETHER_U128)).to_string(),
            "9000 ETH"
        );
    }
}
