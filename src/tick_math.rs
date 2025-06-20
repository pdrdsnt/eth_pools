//! Utility functions for Uniswap V3 tick bitmap and tick index math

use alloy::primitives::{U160, U256, aliases::I24};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick {
    pub tick: I24,
    pub liquidity_net: Option<i128>,
}

/// Normalize a tick by tick spacing (division towards zero)
pub fn normalize_tick(current_tick: I24, tick_spacing: I24) -> I24 {
    current_tick.div_euclid(tick_spacing)
}

pub fn word_index(normalized_tick: I24) -> i16 {
    let divisor = I24::try_from(256).unwrap(); // Infallible for 256
    let quotient = normalized_tick.div_euclid(divisor);
    quotient.as_i16() // Safe: quotient ∈ [-32,768, 32,767]
}
/// Extract initialized tick values from a single bitmap word
pub fn extract_ticks_from_bitmap(bitmap: U256, word_idx: I24, tick_spacing: I24) -> Vec<I24> {
    let mut ticks = Vec::new();
    if bitmap.is_zero() {
        return ticks;
    }
    for bit in 0..256 {
        if bitmap.bit(bit) {
            let normalized =
                (word_idx * I24::try_from(256).unwrap()) + (I24::try_from(bit).unwrap() as I24);
            ticks.push(normalized * tick_spacing);
        }
    }
    ticks
}

pub fn next_left(word: &U256, start: &i16) -> Option<usize> {
    // clamp start to valid range 0..=255
    let mut idx = *start.max(&0_i16).min(&255_i16) as usize;
    // scan forward until we find a set bit or run out of bits
    while idx >= 0 {
        if word.bit(idx) {
            return Some(idx);
        }
        idx -= 1;
    }
    None
}

pub fn next_right(word: &U256, start: &i16) -> Option<usize> {
    // clamp start to valid range 0..=255
    let mut idx = *start.max(&0_i16).min(&255_i16) as usize;
    // scan forward until we find a set bit or run out of bits
    while idx <= 255 {
        if word.bit(idx) {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

/// Given a map of word_index -> bitmap, produce all initialized ticks
pub fn collect_ticks_from_map(
    word_map: &std::collections::HashMap<I24, U256>,
    tick_spacing: I24,
) -> Vec<I24> {
    let mut ticks = Vec::new();
    for (&word_idx, &bitmap) in word_map.iter() {
        ticks.extend(extract_ticks_from_bitmap(bitmap, word_idx, tick_spacing));
    }
    ticks.sort_unstable();
    ticks
}

pub fn price_from_tick(target_tick: I24) -> Option<U160> {
    let max_tick: I24 = I24::try_from(887272).unwrap();
    let abs_tick = target_tick.abs();

    if abs_tick > max_tick {
        eprintln!(
            "[0] Tick {} exceeds maximum allowed (±{})",
            target_tick, max_tick
        );
        return None;
    }

    let mut ratio = if (abs_tick & I24::ONE) != I24::ZERO {
        U256::from_str_radix("255706422905421325395407485534392863200", 10).unwrap()
    } else {
        U256::from(1) << 128
    };

    // Magic numbers are now ordered from highest mask to lowest
    let magic_numbers = [
        (
            I24::try_from(0x80000).unwrap(),
            U256::from_str_radix("366325949420163452428643381347626447728", 10).unwrap(),
        ),
        (
            I24::try_from(0x40000).unwrap(),
            U256::from_str_radix("435319348045928502739365042735923241779", 10).unwrap(),
        ),
        (
            I24::try_from(0x20000).unwrap(),
            U256::from_str_radix("142576269300693600730609870735819320320", 10).unwrap(),
        ),
        (
            I24::try_from(0x10000).unwrap(),
            U256::from_str_radix("366325949420163452428643381347626447728", 10).unwrap(),
        ),
        (
            I24::try_from(0x8000).unwrap(),
            U256::from_str_radix("844815322999501822113930908203125000000", 10).unwrap(),
        ),
        (
            I24::try_from(0x4000).unwrap(),
            U256::from_str_radix("340265210418746478515625000000000000000", 10).unwrap(),
        ),
        (
            I24::try_from(0x2000).unwrap(),
            U256::from_str_radix("215416728668509908758128906250000000000", 10).unwrap(),
        ),
        (
            I24::try_from(0x1000).unwrap(),
            U256::from_str_radix("177803588050028359909546862144531250000", 10).unwrap(),
        ),
        (
            I24::try_from(0x800).unwrap(),
            U256::from_str_radix("170408874814886611515626254292199532339", 10).unwrap(),
        ),
        (
            I24::try_from(0x400).unwrap(),
            U256::from_str_radix("170141183460469231731687303715884105728", 10).unwrap(),
        ),
        (
            I24::try_from(0x200).unwrap(),
            U256::from_str_radix("3868562622766813359059763198240802791", 10).unwrap(),
        ),
        (
            I24::try_from(0x100).unwrap(),
            U256::from_str_radix("29287344681543793554040907002057611822", 10).unwrap(),
        ),
        (
            I24::try_from(0x80).unwrap(),
            U256::from_str_radix("115165952705265534866474743471916972268", 10).unwrap(),
        ),
        (
            I24::try_from(0x40).unwrap(),
            U256::from_str_radix("191204177664095573937843702857003287777", 10).unwrap(),
        ),
        (
            I24::try_from(0x20).unwrap(),
            U256::from_str_radix("234435455086227615880830483505416481938", 10).unwrap(),
        ),
        (
            I24::try_from(0x10).unwrap(),
            U256::from_str_radix("250846047417607353339794883300939388931", 10).unwrap(),
        ),
        (
            I24::try_from(0x8).unwrap(),
            U256::from_str_radix("254322734553735582512512255949976165369", 10).unwrap(),
        ),
        (
            I24::try_from(0x4).unwrap(),
            U256::from_str_radix("255223438104885656517683320344580614584", 10).unwrap(),
        ),
        (
            I24::try_from(0x2).unwrap(),
            U256::from_str_radix("255706422905421325395407485534392863200", 10).unwrap(),
        ),
    ];

    // Iterate from highest mask to lowest
    for (mask, magic) in magic_numbers.iter() {
        if abs_tick & *mask != I24::ZERO {
            // wrap on overflow, then shift down
            let (wrapped, _) = ratio.overflowing_mul(*magic);
            ratio = wrapped >> 128;
        }
    }

    // 3) invert if tick > 0
    if target_tick > I24::ZERO {
        ratio = U256::MAX / ratio;
    }

    // 4) shift down to Q128.96 and round up if any low bits remain
    let shifted = ratio >> 32;
    let sqrt_price_x96_u256 = if ratio & ((U256::ONE << 32) - U256::ONE) != U256::ZERO {
        shifted + U256::ONE
    } else {
        shifted
    };

    // 5) cast to U160
    let sqrt_price_x96 =
        U160::from(sqrt_price_x96_u256);

    Some(sqrt_price_x96)
}
/// Convert a sqrt price Q128.96 to the nearest tick index (I24)
/// Port of Uniswap V3's TickMath.getTickAtSqrtRatio
pub fn tick_from_price(sqrt_price_x96: U160) -> Option<I24> {
    // Define bounds as U256 to avoid u128 overflow
    let min_sqrt = U160::from(4295128739u64);
    let max_sqrt =
        U160::from_str_radix("146144670348521010328727305220398882237871023970342", 10).unwrap();

    if sqrt_price_x96 < min_sqrt || sqrt_price_x96 >= max_sqrt {
        eprintln!("Sqrt price {} out of bounds", sqrt_price_x96);
        return None;
    }

    // Convert to Q128.128 for log calculation
    let ratio: U256 = U256::from(sqrt_price_x96) << 32;

    // Compute log2(ratio)
    let msb = 255 - ratio.leading_zeros();
    let mut log2 = (U256::from(msb) - U256::from(128u8)) << 64;

    let mut r: alloy::primitives::Uint<256, 4> = ratio >> (msb - 127);
    for i in 0..64 {
        r = (r * r) >> 127;
        let f: alloy::primitives::Uint<256, 4> = r >> 128;
        log2 |= f << (63 - i);
        r >>= f;
    }

    // Calculate candidate ticks
    let _tick_low: alloy::primitives::Uint<256, 4> =
        (log2 - U256::from_str_radix("3402992956809132418596140100660247210", 10).unwrap()) >> 128;
    let _tick_up: alloy::primitives::Uint<256, 4> = (log2
        + U256::from_str_radix("291339464771989622907027621153398088495", 10).unwrap())
        >> 128;
    let tick_low = I24::try_from(_tick_low.as_limbs()[0]).unwrap();
    let tick_high = I24::try_from(_tick_up.as_limbs()[0]).unwrap();

    // Choose nearest
    if tick_low == tick_high {
        Some(tick_low)
    } else if price_from_tick(tick_high).unwrap_or(U160::ZERO) <= sqrt_price_x96 {
        Some(tick_high)
    } else {
        Some(tick_low)
    }
}

pub fn compute_amount_possible(
    from0: bool,
    available_liquidity: &u128,
    current_sqrt_price: &U160,
    next_sqrt_price: &U160,
) -> Option<U256> {
    // Q96 = 2^96
    let q96 = U256::ONE << 96;

    // promote everything to U256
    let liq: U256 = U256::from(*available_liquidity);
    let cur: U256 = U256::from(*current_sqrt_price);
    let nxt: U256 = U256::from(*next_sqrt_price);

    if from0 {
        // Δx = L·(√P_next − √P_curr)·Q96 ÷ (√P_curr·√P_next)
        let diff = nxt.checked_sub(cur)?;
        if diff.is_zero() {
            return None;
        }

        // numerator = L * diff * Q96
        let numerator = liq
            .checked_mul(diff)?
            .checked_mul(q96)?;

        // denominator = cur * nxt
        let denominator = cur.checked_mul(nxt)?;
        Some(numerator.checked_div(denominator)?)
    } else {
        // Δy = L·(√P_curr − √P_next) ÷ Q96
        let diff = cur.checked_sub(nxt)?;
        if diff.is_zero() {
            return None;
        }

        let numerator = liq.checked_mul(diff)?;
        Some(numerator.checked_div(q96)?)
    }
}

pub fn compute_price_from0(
    amount: &U256,
    available_liquidity: &u128,
    current_sqrt_price: &U160,
    add: bool,
) -> Option<U256> {
    // Debug prints (optional)
    // println!("Inputs:");
    // println!("  Δx (amount): {}", amount);
    // println!("  L (liquidity): {}", available_liquidity);
    // println!("  √P (current_sqrt_price): {}", current_sqrt_price);

    // Step 1: Compute L << 96 (Q96L)
    let q96L = U160::from(*available_liquidity) << (U160::from(96_u32));
    // println!("Q96L (L << 96): {}", Q96L);

    // Step 2: Compute (L << 96) / √P (scaled_liquidity)
    let scaled_liquidity = U256::from(q96L.checked_div(*current_sqrt_price)?);
    // println!("scaled_liquidity (Q96L / √P): {}", scaled_liquidity);

    // Step 3: Compute denominator = scaled_liquidity ± Δx
    let denominator = if add {
        scaled_liquidity.checked_add(*amount)?
    } else {
        scaled_liquidity.checked_sub(*amount)?
    };
    // println!("denominator (scaled_liquidity ± Δx): {}", denominator);

    // Step 4: Compute new_sqrt_price = Q96L / denominator
    let new_sqrt_price = Q96L.checked_div(denominator)?;
    // println!("new_sqrt_price (Q96L / denominator): {}", new_sqrt_price);

    Some(new_sqrt_price)
}

pub fn compute_price_from1(
    amount: &U256,
    available_liquidity: &u128,
    current_sqrt_price: &U160,
    add: bool,
) -> Option<U256> {
    // For token1, calculate the difference as (current - next)
    let n = (*amount << U256::from(96_u32)).checked_div(*available_liquidity)?;
    // amount_possible = available_liquidity * diff

    Some(if add {
        current_sqrt_price.checked_add(n)?
    } else {
        current_sqrt_price.checked_sub(n)?
    })
}
pub fn update_liquidity(current: U256, net: i128) -> Option<U256> {
    let _net = U256::from(net);

    if net < i128::from(0) {
        current.checked_sub(_net)
    } else {
        current.checked_sub(_net)
    }
}
