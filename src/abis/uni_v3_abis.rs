use alloy::sol;

sol! {
    #[sol(rpc)]
    contract UniV3Pool {
        function slot0() external view returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );
        function ticks(int24 tick) external view returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128,
            int56 tickCumulativeOutside,
            uint160 secondsPerLiquidityOutsideX128,
            uint32 secondsOutside,
            bool initialized
        );  

        function token0() external view returns (address);
        function token1() external view returns (address);
        function fee() external view returns (uint24);
        function tickBitmap(int16 wordPosition) external view returns (uint256);
        function liquidity() external view returns (uint128);
        function tickSpacing() external view returns (int24);
    }
}


