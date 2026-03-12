use anchor_lang::prelude::*;

#[error_code]
pub enum LiquidityError {
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,

    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Invalid token pair")]
    InvalidTokenPair,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Zero amount")]
    ZeroAmount,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Pool already exists")]
    PoolExists,

    #[msg("Invalid fee")]
    InvalidFee,

    #[msg("Insufficient shares")]
    InsufficientShares,

    #[msg("Oracle price error")]
    OracleError,
}
