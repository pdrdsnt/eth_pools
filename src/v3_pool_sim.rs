use std::ops::DerefMut;

use alloy::primitives::{Address, U256, aliases::I24};

use crate::{
    tick_math::{self, Tick},
    trade::Trade,
};

#[derive(Debug)]
pub struct V3PoolSim {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub exchange: String,
    pub version: String,
    pub fee: u32,
    pub active_ticks: Vec<Tick>,
    pub tick_spacing: i32,
    pub liquidity: U256,
    pub x96price: U256,
}

impl V3PoolSim {
    // Private constructor
    pub fn new(
        address: Address,
        fee: u32,
        dex: String,
        version: String,
        token0: Address,
        token1: Address,
        tick_spacing: i32,
        active_ticks: Vec<Tick>,
        liquidity: U256,
        x96price: U256,
    ) -> Self {
        Self {
            address,
            token0,
            token1,
            exchange: dex,
            version,
            active_ticks,
            fee,
            tick_spacing,
            liquidity,
            x96price,
        }
    }
    pub fn trade(&mut self, amount_in: U256, from0: bool) -> Option<Trade> {
        // 1. Fee deduction
        let fee_amount = amount_in
            .checked_mul(U256::from(self.fee))?
            .checked_div(U256::from(1_000_000))?;
        let mut remaining = amount_in.checked_sub(fee_amount)?;

        // 2. Local state
        let mut total_out = U256::ZERO;

        let mut curr_price = self.x96price;

        let current_tick = tick_math::tick_from_price(self.x96price)?;
        let mut next_tick_index = match self
            .active_ticks
            .binary_search_by_key(&current_tick, |t| t.tick)
        {
            Ok(i) => {
                if from0 {
                    if i + 1 >= self.active_ticks.len() {
                        return None;
                    } // No ticks above
                    i + 1
                } else {
                    if i == 0 {
                        return None;
                    } // No ticks below
                    i - 1
                }
            }
            Err(i) => {
                if from0 {
                    if i >= self.active_ticks.len() {
                        return None;
                    } // No ticks above
                    i
                } else {
                    if i == 0 {
                        return None;
                    } // No ticks below
                    i - 1
                }
            }
        };
        let mut curr_liq = self.liquidity;

        // 3. Iterate ticks
        while remaining > U256::ZERO {
            // get target tick price
            let tick = self.active_ticks.get(next_tick_index as usize)?;
            let next_price = tick_math::price_from_tick(tick.tick)?;
            next_tick_index = if from0 {
                next_tick_index.checked_add(1)?
            } else {
                next_tick_index.checked_sub(1)?
            };

            // compute max amount possible to cross this tick
            let possible =
                tick_math::compute_amount_possible(from0, &curr_liq, &curr_price, &next_price)?;

            if remaining < possible {
                // won't cross full tick
                let new_price = if from0 {
                    tick_math::compute_price_from0(&remaining, &curr_liq, &curr_price, true)?
                } else {
                    tick_math::compute_price_from1(&remaining, &curr_liq, &curr_price, true)?
                };

                // compute out
                let delta = if from0 {
                    curr_liq
                        .checked_mul(new_price.checked_sub(curr_price)?)?
                        .checked_div(U256::from(1u128 << 96))?
                } else {
                    let inv_curr = (U256::ONE << U256::from(96_u32))
                        .checked_mul(U256::ONE << 96)?
                        .checked_div(curr_price)?;
                    let inv_new = (U256::ONE << U256::from(96_u32))
                        .checked_mul(U256::ONE << 96)?
                        .checked_div(new_price)?;
                    curr_liq
                        .checked_mul(inv_curr.checked_sub(inv_new)?)?
                        .checked_div(U256::from(1u128 << 96))?
                };

                total_out = total_out.checked_add(delta)?;
                remaining = U256::ZERO;
                curr_price = new_price;
                break;
            }

            // cross entire tick
            let out_cross = if from0 {
                curr_liq
                    .checked_mul(next_price.checked_sub(curr_price)?)?
                    .checked_div(U256::from(1u128 << 96))?
            } else {
                let num = curr_liq.checked_mul(curr_price.checked_sub(next_price)?)?;
                num.checked_div(U256::from(1u128 << 96))?
            };
            total_out = total_out.checked_add(out_cross)?;

            // update liquidity
            if let Some(net) = tick.liquidity_net {
                curr_liq = if from0 {
                    if net > 0 {
                        curr_liq.saturating_add(U256::from(net))
                    } else {
                        curr_liq.saturating_sub(U256::from(-net))
                    }
                } else {
                    if net < 0 {
                        curr_liq.saturating_add(U256::from(net))
                    } else {
                        curr_liq.saturating_sub(U256::from(net))
                    }
                };
            }

            // move pointer
            curr_price = next_price;
            remaining = remaining.checked_sub(possible)?;
        }

        self.liquidity = curr_liq;
        self.x96price = curr_price;

        // build Trade
        Some(Trade {
            dex: self.exchange.clone(),
            version: self.version.clone(),
            fee: self.fee,
            token0: self.token0,
            token1: self.token1,
            pool: self.address,
            from0,
            amount_in,
            amount_out: total_out,
            price_impact: fee_amount,
            fee_amount,
            raw_price: total_out,
        })
    }

    pub fn mint(&mut self, tick_lower: I24, tick_upper: I24, amount: i128) {
        self.update_tick_liquidity(tick_lower, amount);
        self.update_tick_liquidity(tick_upper, -amount);

        let current_tick = tick_math::tick_from_price(self.x96price).unwrap();
        if tick_lower <= current_tick && current_tick < tick_upper {
            self.liquidity = self.liquidity.saturating_add(U256::from(amount as u128));
        }
    }

    pub fn burn(&mut self, tick_lower: I24, tick_upper: I24, amount: i128) {
        self.update_tick_liquidity(tick_lower, -amount);

        self.update_tick_liquidity(tick_upper, amount);

        let current_tick = tick_math::tick_from_price(self.x96price).unwrap();
        if tick_lower <= current_tick && current_tick < tick_upper {
            self.liquidity = self.liquidity.saturating_sub(U256::from(amount as u128));
        }
    }

    pub fn update_tick_liquidity(&mut self, tick: I24, amount: i128) {
        if self.active_ticks.is_empty() {
            return; // nothing to do, or maybe insert if in range
        }

        let first = self.active_ticks.first().unwrap().tick;
        let last = self.active_ticks.last().unwrap().tick;

        // out of range: ignore
        if tick < first || tick > last {
            return;
        }

        match self.active_ticks.binary_search_by_key(&tick, |t| t.tick) {
            Ok(pos) => {
                // found existing tick
                let tick_ref: &mut Tick = &mut self.active_ticks[pos];
                if let Some(liq) = tick_ref.liquidity_net.as_mut() {
                    *liq += amount;

                    if *liq == i128::from(0) {
                        self.active_ticks.remove(pos);
                    }
                }
            }
            Err(pos) => {
                // not found, insert at position pos
                self.active_ticks.insert(
                    pos,
                    Tick {
                        tick,
                        liquidity_net: Some(amount),
                    },
                );
            }
        }
    }
}
