use anchor_lang::prelude::*;

#[error_code]
pub enum BridgeError {
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,

    #[msg("Insufficient user balance")]
    InsufficientBalance,

    #[msg("Invalid token pair")]
    InvalidTokenPair,

    #[msg("Invalid fee basis points")]
    InvalidFeeBps,

    #[msg("Bridge is paused")]
    BridgePaused,

    #[msg("Unauthorized vault authority")]
    UnauthorizedVaultAuthority,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Vault not initialized")]
    VaultNotInitialized,

    #[msg("Invalid collateral ratio")]
    InvalidCollateralRatio,

    #[msg("Wrapped token mismatch")]
    WrappedTokenMismatch,

    #[msg("Transfer amount too large")]
    TransferAmountTooLarge,
}
