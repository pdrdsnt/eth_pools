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

    use alloy::{
        primitives::{Address, U256},
        transports::http::reqwest::Url,
    };
    use alloy_provider::ProviderBuilder;

    use super::*;

    #[tokio::test]
    async fn v3_src() {
        let base = ProviderBuilder::new()
            .connect_http(Url::from_str("https://binance.llamarpc.com").unwrap());

        let provider = base;
        //other bsc v3 to test
        //0x28dF0835942396B7a1b7aE1cd068728E6ddBbAfD
        //0x0f338Ec12d3f7C3D77A4B9fcC1f95F3FB6AD0EA6
        let v3 = v3_pool_src::V3PoolSrc::new(
            Address::from_str("0x0f338Ec12d3f7C3D77A4B9fcC1f95F3FB6AD0EA6").unwrap(),
            provider,
        )
        .await;

        let mut new_v3 = v3.unwrap();
        
        let mut current_tick = new_v3.current_tick;
        let mut current_price = new_v3.x96price;

        println!("slot0 tick {}",current_tick);        
        
        let mut my_price = tick_math::price_from_tick(current_tick).unwrap();
        println!("calc price {}", my_price);
        println!("slot0 price {}", current_price);


        /*
        let mut f = false;
        for tick in &new_v3.active_ticks {
            let c = if tick.tick >= new_v3.current_tick && !f {
                f = true;
                "<<<<"
            } else {
                ""
            };
            println!("{} - {:?} {}", tick.tick, tick.liquidity_net, c);
        }

        if let Some(sim) = new_v3.trade(U256::ONE << 64, false) {
            println!("simulating trade: {:?} ", sim);
        } else {
            println!("simulation failed");
        }*/
    }
}
