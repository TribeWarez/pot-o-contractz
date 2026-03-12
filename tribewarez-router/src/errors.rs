use anchor_lang::prelude::*;

#[error_code]
pub enum RouterError {
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,

    #[msg("Invalid swap path")]
    InvalidSwapPath,

    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Invalid route")]
    InvalidRoute,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Invalid token")]
    InvalidToken,

    #[msg("Route too long")]
    RouteTooLong,

    #[msg("Zero amount")]
    ZeroAmount,
}
