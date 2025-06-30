use std::collections::HashMap;

use alloy::primitives::aliases::U24;
use alloy::primitives::{Address, aliases::I24};
use alloy::primitives::{U256, U512};

use alloy_provider::{RootProvider, fillers::FillProvider};

use alloy_provider::utils::JoinedRecommendedFillers;

use crate::trade::Trade;
use crate::{
    UniV3Pool::UniV3PoolInstance,
    tick_math::{self, Tick},
};

type Rpc = FillProvider<JoinedRecommendedFillers, RootProvider>;
type PoolContract = UniV3PoolInstance<Rpc>;

#[derive(Debug)]
pub struct V3PoolSrc {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: U24,
    pub current_tick: I24,
    pub active_ticks: Vec<Tick>,
    pub bitmap: HashMap<i16, U256>,
    pub tick_spacing: I24,
    pub liquidity: U256,
    pub x96price: U256,
    pub contract: PoolContract,
}
impl V3PoolSrc {
    pub async fn new(address: Address, provider: Rpc) -> Result<Self, anyhow::Error> {
        let contract = UniV3PoolInstance::new(address, provider);

        let tick_spacing = contract.tickSpacing().call().await?;
        let slot0_return = contract.slot0().call().await?;

        println!("slot0 tick {:?}", slot0_return.tick);
        println!("slot0 price {:?}", slot0_return.sqrtPriceX96);

        let liquidity = U256::from(contract.liquidity().call().await?);
        println!("slot0 liquidity {}", liquidity);
        let fee = contract.fee().call().await?;
        let token0 = contract.token0().call().await?;
        let token1 = contract.token1().call().await?;
        let x96price = U256::from(slot0_return.sqrtPriceX96);
        let mut bitmap: HashMap<i16, U256> = HashMap::new();
        let current_tick = slot0_return.tick.clone();
        let ticks =
            V3PoolSrc::update_ticks(&mut bitmap, current_tick, tick_spacing, 5, &contract).await;
        Ok(Self {
            address,
            token0,
            token1,
            fee,
            current_tick: slot0_return.tick,
            active_ticks: ticks,
            bitmap: bitmap,
            tick_spacing: tick_spacing,
            liquidity,
            x96price,
            contract,
        })
    }

    pub async fn update_ticks(
        bitmap: &mut HashMap<i16, U256>,
        start: I24,
        tick_spacing: I24,
        range: usize,
        contract: &PoolContract,
    ) -> Vec<Tick> {
        let mut r: Vec<I24> =
            V3PoolSrc::right_ticks(bitmap, start, tick_spacing, range, contract).await;
        let mut l: Vec<I24> =
            V3PoolSrc::left_ticks(bitmap, start, tick_spacing, range, contract).await;

        l.reverse();
        l.append(&mut r);

        let mut ticks = Vec::new();
        for tick in l {
            if let Ok(fut) = contract.ticks(tick).call().await {
                ticks.push(Tick {
                    tick: tick,
                    liquidity_net: Some(fut.liquidityNet),
                });
            } else {
                ticks.push(Tick {
                    tick: tick,
                    liquidity_net: None,
                });
            }
        }

        ticks
    }

    pub async fn right_ticks(
        bitmap: &mut HashMap<i16, U256>,
        start: I24,
        tick_spacing: I24,
        range: usize,
        contract: &PoolContract,
    ) -> Vec<I24> {
        let mut active_ticks = Vec::<I24>::with_capacity(range);

        let normalized_tick = tick_math::normalize_tick(start, tick_spacing);

        let mut current_pos = normalized_tick % I24::try_from(256).unwrap();
        let mut current_word_idx = tick_math::word_index(normalized_tick);
        let mut current_word_global = current_word_idx * 256;

        while active_ticks.len() < range {
            if let Some(c_word) = bitmap.get(&current_word_idx) {
                if let Some(v) = tick_math::next_right(&*c_word, &current_pos.low_i16()) {
                    let tick = (I24::try_from(current_word_global).unwrap()
                        + I24::try_from(v).unwrap())
                        * tick_spacing;
                    current_pos = I24::try_from(v + 1).unwrap();
                    active_ticks.push(tick);
                } else {
                    current_pos = I24::ZERO;
                    current_word_idx += 1;
                    current_word_global = current_word_idx * 256;
                }
            } else {
                if let Ok(c_word) = contract.tickBitmap(current_word_idx).call().await {
                    bitmap.insert(current_word_idx, c_word);
                } else {
                    break;
                }
            }
        }

        active_ticks
    }

    pub async fn left_ticks(
        bitmap: &mut HashMap<i16, U256>,
        start: I24,
        tick_spacing: I24,
        range: usize,
        contract: &PoolContract,
    ) -> Vec<I24> {
        let mut active_ticks = Vec::<I24>::with_capacity(range);

        let normalized_tick = tick_math::normalize_tick(start, tick_spacing);

        let mut current_pos = normalized_tick % I24::try_from(256).unwrap();
        let mut current_word_idx = tick_math::word_index(normalized_tick);
        let mut current_word_global = current_word_idx * 256;

        while active_ticks.len() < range {
            if let Some(c_word) = bitmap.get(&current_word_idx) {
                if let Some(v) = tick_math::next_left(&*c_word, &current_pos.low_i16()) {
                    let tick = (I24::try_from(current_word_global).unwrap()
                        + I24::try_from(v).unwrap())
                        * tick_spacing;
                    current_pos = I24::try_from(v - 1).unwrap();
                    active_ticks.push(tick);
                } else {
                    current_pos = I24::try_from(256_i16).unwrap();
                    current_word_idx -= 1;
                    current_word_global = current_word_idx * 256;
                }
            } else {
                if let Ok(c_word) = contract.tickBitmap(current_word_idx).call().await {
                    bitmap.insert(current_word_idx, c_word);
                } else {
                    break;
                }
            }
        }

        active_ticks
    }

    pub fn trade(&mut self, amount_in: U256, from0: bool) -> Option<Trade> {
        // 1. Fee deduction
        println!("remaining before fee {}", amount_in);
        let fee_amount = amount_in
            .checked_mul(U256::from(self.fee))?
            .checked_div(U256::from(1_000_000))?;
        let mut remaining = amount_in.checked_sub(fee_amount)?;
        println!("remaining after fee {}", remaining);
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
        println!("next tick: {}", next_tick_index);
        let mut curr_liq = self.liquidity;

        // 3. Iterate ticks

        while remaining > U256::ZERO {
            // get target tick price

            println!("loop step");
            let next_tick = self.active_ticks.get(next_tick_index as usize)?;
            let next_price = tick_math::price_from_tick(next_tick.tick)?;

            let current_price_v = tick_math::price_from_tick(current_tick)?;

            println!("price next {}", next_price);
            println!("s0 price curr {}", curr_price);
            println!("my price curr {}", current_price_v);

            println!("next tick {}", next_tick.tick);
            println!("my curr tick {}", tick_math::tick_from_price(curr_price)?);
            println!("s0 curr tick {}", current_tick);
            let old_tick_index = next_tick_index;
            next_tick_index = if from0 {
                next_tick_index.checked_add(1)?
            } else {
                next_tick_index.checked_sub(1)?
            };

            // compute max amount possible to cross this tick
            let possible =
                tick_math::compute_amount_possible(from0, &curr_liq, &curr_price, &next_price)?;

            // **DEBUG PRINT**
            println!(
                "[DEBUG] tick_index={} → next_index={} | curr_price={} | next_price={} | \
         curr_liq={} | remaining={} | possible_to_cross={}",
                old_tick_index,
                next_tick_index,
                curr_price,
                next_price,
                curr_liq,
                remaining,
                possible
            );

            if remaining < possible {
                // won't cross full tick
                let new_price = if from0 {
                    tick_math::compute_price_from0(&remaining, &curr_liq, &curr_price, true)?
                } else {
                    tick_math::compute_price_from1(&remaining, &curr_liq, &curr_price, true)?
                };

                // **DEBUG PRINT**
                println!(
                    "[DEBUG] crossing partial tick: remaining={} < possible={}, new_price={}",
                    remaining, possible, new_price
                );

                let u512_curr_price = U512::from(curr_price);
                let u512_curr_liq = U512::from(curr_liq);

                // compute out
                let delta = if from0 {
                    let price_diff = u512_curr_price.checked_sub(U512::from(new_price))?;
                    println!("diff {}", price_diff);
                    u512_curr_liq
                        .checked_mul(price_diff)?
                        .checked_div(U512::ONE << 96)?
                } else {
                    let inv_curr = (U512::ONE << U512::from(96_u32))
                        .checked_mul(U512::ONE << 96)?
                        .checked_div(u512_curr_price)?;
                    let inv_new = (U512::ONE << U512::from(96_u32))
                        .checked_mul(U512::ONE << 96)?
                        .checked_div(U512::from(new_price))?;
                    u512_curr_liq
                        .checked_mul(inv_curr.checked_sub(inv_new)?)?
                        .checked_div(U512::from(1u128 << 96))?
                };

                println!("delta {}", delta);

                total_out = total_out.checked_add(U256::from(delta))?;
                remaining = U256::ZERO;
                curr_price = U256::from(new_price);

                // **DEBUG PRINT**
                println!(
                    "[DEBUG] partial-cross: delta_out={} | total_out={} | curr_price(updated)={} | remaining=0",
                    delta, total_out, curr_price
                );

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

            // **DEBUG PRINT**
            println!(
                "[DEBUG] full-cross: out_cross={} | total_out(before_liq_update)={} ",
                out_cross,
                total_out - out_cross
            );

            // update liquidity
            if let Some(net) = next_tick.liquidity_net {
                let old_liq = curr_liq;
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
                // **DEBUG PRINT**
                println!(
                    "[DEBUG] liquidity_net={} | liquidity: {} → {}",
                    net, old_liq, curr_liq
                );
            }

            // move pointer
            curr_price = next_price;
            remaining = remaining.checked_sub(possible)?;

            // **DEBUG PRINT**
            println!(
                "[DEBUG] end-of-iteration: curr_price={} | remaining={} | total_out={}\n",
                curr_price, remaining, total_out
            );
        }

        self.liquidity = curr_liq;
        self.x96price = curr_price;

        // build Trade
        Some(Trade {
            fee: self.fee,
            token0: self.token0,
            token1: self.token1,
            pool: self.address,
            from0,
            amount_in,
            amount_out: total_out,
        })
    }
}
