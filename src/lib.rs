mod v3_pool_src;

include!("abis/uni_v3_abis.rs");

mod err;
mod tick_math;
mod trade;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{primitives::{Address, U256}, transports::http::reqwest::Url};
    use alloy_provider::ProviderBuilder;

    use super::*;

    #[tokio::test]
    async fn v3_src() {
        let base = ProviderBuilder::new()
            .connect_http(Url::from_str("https://binance.llamarpc.com").unwrap());

        let provider = base;

        let v3 = v3_pool_src::V3PoolSrc::new(
            Address::from_str("0xFe4fe5B4575c036aC6D5cCcFe13660020270e27A").unwrap(),
            provider,
        )
        .await;
        
        let mut new_v3 = v3.unwrap();
        let mut f = false;
        for tick in &new_v3.active_ticks {
            let c = if tick.tick >= new_v3.current_tick && !f { f = true; "<<<<" } else {""};
            println!("{} - {:?} {}", tick.tick, tick.liquidity_net, c);
        }

        if let Some(sim) = new_v3.trade(U256::ONE << 12, true) {
            println!("simulating trade: {:?} ", sim);
        }else {
            println!("simulation failed");
        }

        

    }
}
