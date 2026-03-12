//! Token program errors

use anchor_lang::prelude::*;

#[error_code]
pub enum TokenError {
    #[msg("Supply cap exceeded")]
    SupplyCapExceeded,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Invalid decimals")]
    InvalidDecimals,

    #[msg("Invalid mint authority")]
    InvalidMintAuthority,

    #[msg("Invalid freeze authority")]
    InvalidFreezeAuthority,

    #[msg("Account is frozen")]
    AccountFrozen,

    #[msg("Invalid owner")]
    InvalidOwner,

    #[msg("Supply overflow")]
    SupplyOverflow,

    #[msg("Unauthorized operation")]
    Unauthorized,

    #[msg("Invalid token amount")]
    InvalidTokenAmount,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
