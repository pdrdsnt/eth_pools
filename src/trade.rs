use alloy::primitives::{Address, U256};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord,)]
pub struct Trade {
    pub dex: String,
    pub version: String,
    pub fee: u32,
    pub token0: Address,
    pub token1: Address,
    pub pool: Address,
    pub from0: bool,
    pub amount_in: U256,
    pub amount_out: U256,
    pub price_impact: U256,
    pub fee_amount: U256,
    pub raw_price: U256,
}
