use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::asset::Amount;
use crate::Address;

/// ERC-20 standard token.
#[derive(Debug, Deserialize, Serialize, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Erc20 {
    pub token_contract: Address,
    pub amount: Amount,
}

impl Display for Erc20 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
