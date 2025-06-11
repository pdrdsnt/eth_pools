use crate::v2_pool_sim::V2PoolSim;
use alloy::{
    network::Ethereum, primitives::{Address, Bytes, U256}, sol, sol_types::SolCall
};
use alloy_provider::{fillers::FillProvider, utils::JoinedRecommendedFillers, RootProvider};

sol!{
        interface IUniswapV2Pair {
            function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        }
    }

#[derive(Debug)]
pub struct V2PoolSrc {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub exchange: String,
    pub version: String,
    pub fee: u32,
    pub reserves0: U256,
    pub reserves1: U256,
}

impl V2PoolSrc {
    // Private constructor
    async fn new(
        exchange: String,
        version: String,
        fee: u32,
        address: Address,
        token0: Address,
        token1: Address,
    ) -> Self {
        let mut instance = V2PoolSrc {
            exchange,
            version,
            fee,
            address,
            token0,
            token1,
            reserves0: U256::ZERO,
            reserves1: U256::ZERO,
        };
        instance.update().await;
        instance
    }

    pub async fn update(&mut self, provider: &FillProvider<JoinedRecommendedFillers, RootProvider<Ethereum>>) {
        let call = IUniswapV2Pair::getReservesCall {};
        let calldata: Bytes = call.abi_encode().into();
    }

    pub async fn into_sim(&self) -> V2PoolSim {
        V2PoolSim::new(
            self.exchange.clone(),
            self.version.clone(),
            self.fee.clone(),
            self.address.clone(),
            self.token0.clone(),
            self.token1.clone(),
            self.reserves0,
            self.reserves1,
        )
    }
}
