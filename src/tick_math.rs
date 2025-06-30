//////////! Utility functions for Uniswap V3 tick bitmap and tick index math

use alloy::primitives::{I256, U256, U512, aliases::I24};

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

pub fn price_from_tick(target_tick: I24) -> Option<U256> {
    println!("calculating prince fomr tick: {}", target_tick);
    let max_tick: I24 = I24::try_from(887272).unwrap();
    let abs_tick = target_tick.abs();

    if abs_tick > max_tick {
        eprintln!(
            "[0] Tick {} exceeds maximum allowed (±{})",
            target_tick, max_tick
        );
        return None;
    }

    let mut sqrt_price_x128 = if (abs_tick & I24::ONE) != I24::ZERO {
        U512::from_str_radix("fffcb933bd6fad37aa2d162d1a594001", 16).unwrap()
    } else {
        U512::from(1) << 128
    };

    let magic_numbers = [
        // mask 0x1  (handled in your `sqrt_price_x128 = …` init)
        (
            0x2,
            U512::from_str_radix("fff97272373d413259a46990580e213a", 16).unwrap(),
        ),
        (
            0x4,
            U512::from_str_radix("fff2e50f5f656932ef12357cf3c7fdcc", 16).unwrap(),
        ),
        (
            0x8,
            U512::from_str_radix("ffe5caca7e10e4e61c3624eaa0941cd0", 16).unwrap(),
        ),
        (
            0x10,
            U512::from_str_radix("ffcb9843d60f6159c9db58835c926644", 16).unwrap(),
        ),
        (
            0x20,
            U512::from_str_radix("ff973b41fa98c081472e6896dfb254c0", 16).unwrap(),
        ),
        (
            0x40,
            U512::from_str_radix("ff2ea16466c96a3843ec78b326b52861", 16).unwrap(),
        ),
        (
            0x80,
            U512::from_str_radix("fe5dee046a99a2a811c461f1969c3053", 16).unwrap(),
        ),
        (
            0x100,
            U512::from_str_radix("fcbe86c7900a88aedcffc83b479aa3a4", 16).unwrap(),
        ),
        (
            0x200,
            U512::from_str_radix("f987a7253ac413176f2b074cf7815e54", 16).unwrap(),
        ),
        (
            0x400,
            U512::from_str_radix("f3392b0822b70005940c7a398e4b70f3", 16).unwrap(),
        ),
        (
            0x800,
            U512::from_str_radix("e7159475a2c29b7443b29c7fa6e889d9", 16).unwrap(),
        ),
        (
            0x1000,
            U512::from_str_radix("d097f3bdfd2022b8845ad8f792aa5825", 16).unwrap(),
        ),
        (
            0x2000,
            U512::from_str_radix("a9f746462d870fdf8a65dc1f90e061e5", 16).unwrap(),
        ),
        (
            0x4000,
            U512::from_str_radix("70d869a156d2a1b890bb3df62baf32f7", 16).unwrap(),
        ),
        (
            0x8000,
            U512::from_str_radix("31be135f97d08fd981231505542fcfa6", 16).unwrap(),
        ),
        (
            0x10000,
            U512::from_str_radix("9aa508b5b7a84e1c677de54f3e99bc9", 16).unwrap(),
        ),
        (
            0x20000,
            U512::from_str_radix("5d6af8dedb81196699c329225ee604", 16).unwrap(),
        ),
        (
            0x40000,
            U512::from_str_radix("2216e584f5fa1ea926041bedfe98", 16).unwrap(),
        ),
        (
            0x80000,
            U512::from_str_radix("48a170391f7dc42444e8fa2", 16).unwrap(),
        ),
    ];

    // Iterate from highest mask to lowest
    for (mask, magic) in magic_numbers.iter() {
        if abs_tick & I24::try_from(*mask).unwrap() != I24::ZERO {
            // wrap on overflow, then shift down
            let (wrapped, _) = sqrt_price_x128.overflowing_mul(*magic);
            sqrt_price_x128 = wrapped >> 128;
            println!("price {}", sqrt_price_x128);
            println!("step {}", mask);
        }
    }
    let mut p256 = U256::from(sqrt_price_x128);

    if target_tick > I24::ZERO {
        if sqrt_price_x128.is_zero() {
            return None; // Should ideally not happen if initial sqrt_price_x128 is non-zero
        }
        p256 = U256::MAX.checked_div(p256).unwrap();
    }

    // 4) shift down to Q128.96 and round up if any low bits remain

    let shifted = p256 >> 32;
    let sqrt_price_x96_u256 = if p256 & ((U256::ONE << 32) - U256::ONE) != U256::ZERO {
        shifted + U256::ONE
    } else {
        shifted
    };

    // 5) cast to U160
    // let sqrt_price_x96 = U160::from(sqrt_price_x96_u256);

    Some(sqrt_price_x96_u256)
}
// Convert a sqrt price Q128.96 to the nearest tick index (I24)
/// Port of Uniswap V3's TickMath.getTickAtSqrtRatio
pub fn tick_from_price(sqrt_price_x96: U256) -> Option<I24> {
    // Define bounds as U256 to avoid u128 overflow
    let min_sqrt = U256::from(4295128739u64);
    let max_sqrt =
        U256::from_str_radix("1461446703485210103287273052203988822378723970342", 10).unwrap();

    if sqrt_price_x96 < min_sqrt || sqrt_price_x96 >= max_sqrt {
        eprintln!("Sqrt price {} out of bounds", sqrt_price_x96);
        return None;
    }

    // Convert to Q128.128 for log calculation
    let sqrroot_price_x128: U256 = sqrt_price_x96 << 32;

    // Compute log2(sqrroot_price_x128)
    let msb = 255 - sqrroot_price_x128.leading_zeros();
    println!("most significant bit {}", msb);
    let mut log2: I256 = (I256::try_from(msb).unwrap() - I256::try_from(128u8).unwrap()) << 64;

    println!("log2 before loop: {}", log2);
    let mut r = if msb >= 128 {
        sqrroot_price_x128 >> (msb - 127)
    } else {
        sqrroot_price_x128 << (127 - msb)
    };
    println!("r: {}", r);
    for i in 0..14 {
        r = (r * r) >> 127;
        let f: U256 = r >> 128;
        let shift = 63 - i;
        let a: U256 = f << shift;
        log2 |= I256::from(a);
        r >>= f;
    }

    println!("log2 after loop: {}", log2);
    let log_sqrt10001 = log2 * I256::try_from("255738958999603826347141").unwrap();
    I256::try_from("255738958999603826347141").unwrap();

    println!("log2 afet / 2^128: {}", log_sqrt10001);
    let low = (log_sqrt10001 - I256::try_from("3402992956809132418596140100660247210").unwrap())
        >> 128_u32;
    let high = (log_sqrt10001 + I256::try_from("291339464771989622907027621153398088495").unwrap())
        >> 128_u32;
    println!("high {}", high);
    println!("low {}", low);

    // Calculate candidate ticks
    let tick_low: I24 = I24::from(low);
    let tick_high: I24 = I24::from(high);

    println!("low: {} = high: {}", tick_low, tick_high);

    let result = if tick_high == tick_low {
        tick_high
    } else {
        if price_from_tick(tick_high)? >= sqrt_price_x96 {
            tick_high
        } else {
            tick_low
        }
    };
    Some(result)
}
pub fn compute_amount_possible(
    from0: bool,
    available_liquidity: &U256,
    current_sqrt_price: &U256,
    next_sqrt_price: &U256,
) -> Option<U256> {
    println!("computing amount possible");
    // Q96 = 2^96
    let q96: U512 = U512::ONE << 96;

    //promote everything to U512
    let liq: U512 = U512::from(*available_liquidity);
    let cur: U512 = U512::from(*current_sqrt_price);
    let nxt: U512 = U512::from(*next_sqrt_price);

    if from0 {
        // Δx = L·(√P_next − √P_curr)·Q96 ÷ (√P_curr·√P_next)
        println!("from 0");
        println!("next price {}", next_sqrt_price);
        println!("current price {}", current_sqrt_price);

        let diff = nxt.checked_sub(cur)?;
        println!("passed");

        if diff.is_zero() {
            println!("diff is zezo");
            return None;
        }

        println!("q 96 {}", q96);
        println!("available liquidity {}", available_liquidity);
        println!("diff {}", diff);
        // numerator = L * diff * Q96
        let impact = liq.checked_mul(diff)?;
        let numerator: U512 = U512::from(impact).checked_mul(q96)?;
        println!("numerator {}", numerator);
        // denominator = cur * nxt
        let denominator = cur.checked_mul(nxt)?;
        println!("denominator {}", denominator);

        let res = U256::from(numerator.checked_div(denominator)?);

        Some(res)
    } else {
        println!("from 1");
        // Δy = L·(√P_curr − √P_next) ÷ Q96
        let diff = cur.checked_sub(nxt)?;
        println!("diff {}", diff);
        if diff.is_zero() {
            println!("diff is zero");
            return None;
        }

        let numerator = liq.checked_mul(diff)?;
        println!("numerator {}", numerator);
        Some(U256::from(numerator.checked_div(q96)?))
    }
}
/// Given Δy (token1 amount) and liquidity L, compute the next √P
pub fn compute_price_from0(
    amount: &U256,
    available_liquidity: &U256,
    current_sqrt_price: &U256,
    add: bool,
) -> Option<U256> {
    // Debug prints (optional)
    // println!("Inputs:");
    // println!("  Δx (amount): {}", amount);
    // println!("  L (liquidity): {}", available_liquidity);
    // println!("  √P (current_sqrt_price): {}", current_sqrt_price);
    // Step 1: Compute L << 96 (Q96L)
    let q96_l = *available_liquidity << (U256::from(96_u32));
    // println!("Q96L (L << 96): {}", Q96L);

    // Step 2: Compute (L << 96) / √P (scaled_liquidity)
    let scaled_liquidity = q96_l.checked_div(U256::from(*current_sqrt_price))?;
    // println!("scaled_liquidity (Q96L / √P): {}", scaled_liquidity);

    // Step 3: Compute denominator = scaled_liquidity ± Δx
    let denominator = if add {
        scaled_liquidity.checked_add(*amount)?
    } else {
        scaled_liquidity.checked_sub(*amount)?
    };
    // println!("denominator (scaled_liquidity ± Δx): {}", denominator);

    // Step 4: Compute new_sqrt_price = Q96L / denominator
    let new_sqrt_price = q96_l.checked_div(denominator)?;
    // println!("new_sqrt_price (Q96L / denominator): {}", new_sqrt_price);

    Some(new_sqrt_price)
} // Given Δy (token1 amount) and liquidity L, compute the next √P
/// note: everything in Q96 fixed‐point (i.e. <<96) internally
pub fn compute_price_from1(
    amount: &U256,
    available_liquidity: &U256,
    current_sqrt_price: &U256,
    add: bool,
) -> Option<U256> {
    // Q96 = 2^96
    let q96 = U256::ONE << 96;
    // 1) Scale amount into Q96:   Δy * Q96
    let dy_q96 = amount.checked_mul(q96)?;
    // 2) Divide by liquidity:    Δ√P = (Δy·Q96) / L
    let liquidity_u256 = U256::from(*available_liquidity);
    let delta_sqrt = dy_q96.checked_div(liquidity_u256)?;
    // 3) Apply to current √P
    let cur: U256 = U256::from(*current_sqrt_price);
    let next = if add {
        cur.checked_add(delta_sqrt)?
    } else {
        cur.checked_sub(delta_sqrt)?
    };
    Some(next)
}
pub fn update_liquidity(current_liquidity: U256, liquidity_net: i128) -> Option<U256> {
    if liquidity_net < 0 {
        // If liquidity_net is negative, it means liquidity is removed.
        // We need to subtract the absolute value of liquidity_net.
        let abs_net = U256::from(liquidity_net.abs() as u128); // Convert abs(i128) to u128 then U256
        current_liquidity.checked_sub(abs_net)
    } else {
        // If liquidity_net is positive or zero, it means liquidity is added.
        let pos_net = U256::from(liquidity_net as u128); // Convert positive i128 to u128 then U256
        current_liquidity.checked_add(pos_net)
    }
}
