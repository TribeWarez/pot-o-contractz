/// Swap pricing and fee calculation service.
pub mod swap_calculator;

// Re-export main types for convenience
pub use swap_calculator::{
    SimpleSwapCalculator, SwapCalculator, SwapError, SwapQuote, SwapResult, TensorSwapCalculator,
};
