use alloy::primitives::{aliases::U24, Address, U256};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord,)]
pub struct Trade {
    pub fee: U24,
    pub token0: Address,
    pub token1: Address,
    pub pool: Address,
    pub from0: bool,
    pub amount_in: U256,
    pub amount_out: U256,
}
