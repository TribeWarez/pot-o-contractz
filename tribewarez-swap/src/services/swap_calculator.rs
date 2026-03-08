use anchor_lang::prelude::*;

/// Result type for swap operations.
pub type SwapResult<T> = Result<T, SwapError>;

/// Swap-specific errors.
#[derive(Debug, Clone, Copy)]
pub enum SwapError {
    InvalidAmount,
    InsufficientLiquidity,
    SlippageExceeded,
    MathOverflow,
    InvalidPrice,
    PoolUninitialized,
}

/// Swap pricing and fee information.
#[derive(Clone, Copy)]
pub struct SwapQuote {
    pub amount_out: u64,
    pub swap_fee: u64,
    pub protocol_fee: u64,
    pub lp_fee: u64,
    pub price_impact: f64, // 0.0 to 1.0
}

/// Trait for AMM swap calculations.
///
/// Implements constant product formula: x * y = k
/// with fee-based routing and slippage control.
pub trait SwapCalculator {
    /// Calculate output amount for a given input.
    fn calculate_swap(
        &self,
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> SwapResult<SwapQuote>;

    /// Calculate input needed for desired output.
    fn calculate_reverse_swap(
        &self,
        amount_out: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> SwapResult<u64>;

    /// Validate that output meets minimum requirements (slippage check).
    fn validate_slippage(&self, actual_out: u64, minimum_out: u64) -> SwapResult<()>;

    /// Calculate liquidity provider fees.
    fn calculate_lp_fees(&self, swap_amount: u64) -> u64;

    /// Calculate price impact of a trade.
    fn calculate_price_impact(
        &self,
        amount_in: u64,
        amount_out: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> f64;
}

/// Simple swap calculator (v0.1.x compatible).
///
/// Standard constant product AMM with flat fees.
pub struct SimpleSwapCalculator {
    swap_fee_bps: u64,      // Basis points: 30 = 0.30%
    protocol_fee_bps: u64,
}

impl SimpleSwapCalculator {
    pub fn new(swap_fee_bps: u64, protocol_fee_bps: u64) -> Self {
        SimpleSwapCalculator {
            swap_fee_bps,
            protocol_fee_bps,
        }
    }

    pub fn default_fees() -> Self {
        SimpleSwapCalculator {
            swap_fee_bps: 30,
            protocol_fee_bps: 5,
        }
    }
}

impl SwapCalculator for SimpleSwapCalculator {
    fn calculate_swap(
        &self,
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> SwapResult<SwapQuote> {
        if amount_in == 0 {
            return Err(SwapError::InvalidAmount);
        }

        // Calculate fees
        let swap_fee = (amount_in as u128 * self.swap_fee_bps as u128 / 10000) as u64;
        let protocol_fee = (amount_in as u128 * self.protocol_fee_bps as u128 / 10000) as u64;
        let lp_fee = swap_fee.saturating_sub(protocol_fee);

        // Amount after fees
        let amount_in_after_fee = amount_in.saturating_sub(swap_fee);

        // x * y = k formula
        let k = (reserve_in as u128).checked_mul(reserve_out as u128)
            .ok_or(SwapError::MathOverflow)?;

        let new_reserve_in = (reserve_in as u128)
            .checked_add(amount_in_after_fee as u128)
            .ok_or(SwapError::MathOverflow)?;

        let new_reserve_out = k.checked_div(new_reserve_in)
            .ok_or(SwapError::MathOverflow)? as u64;

        let amount_out = reserve_out.saturating_sub(new_reserve_out);

        if amount_out == 0 {
            return Err(SwapError::InsufficientLiquidity);
        }

        let price_impact = (amount_in as f64) / ((reserve_in as f64) + (amount_in as f64));

        Ok(SwapQuote {
            amount_out,
            swap_fee,
            protocol_fee,
            lp_fee,
            price_impact,
        })
    }

    fn calculate_reverse_swap(
        &self,
        amount_out: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> SwapResult<u64> {
        if amount_out >= reserve_out {
            return Err(SwapError::InsufficientLiquidity);
        }

        let k = (reserve_in as u128).checked_mul(reserve_out as u128)
            .ok_or(SwapError::MathOverflow)?;

        let new_reserve_out = (reserve_out as u128)
            .checked_sub(amount_out as u128)
            .ok_or(SwapError::MathOverflow)?;

        let new_reserve_in = k.checked_div(new_reserve_out)
            .ok_or(SwapError::MathOverflow)? as u64;

        let amount_in_before_fee = new_reserve_in.saturating_sub(reserve_in);
        let amount_in = (amount_in_before_fee as u128)
            .checked_mul(10000)
            .ok_or(SwapError::MathOverflow)?
            .checked_div((10000u128).saturating_sub(self.swap_fee_bps as u128))
            .ok_or(SwapError::MathOverflow)? as u64;

        Ok(amount_in)
    }

    fn validate_slippage(&self, actual_out: u64, minimum_out: u64) -> SwapResult<()> {
        if actual_out < minimum_out {
            return Err(SwapError::SlippageExceeded);
        }
        Ok(())
    }

    fn calculate_lp_fees(&self, swap_amount: u64) -> u64 {
        let total_fee = (swap_amount as u128 * self.swap_fee_bps as u128 / 10000) as u64;
        let protocol_fee = (swap_amount as u128 * self.protocol_fee_bps as u128 / 10000) as u64;
        total_fee.saturating_sub(protocol_fee)
    }

    fn calculate_price_impact(
        &self,
        amount_in: u64,
        _amount_out: u64,
        reserve_in: u64,
        _reserve_out: u64,
    ) -> f64 {
        (amount_in as f64) / ((reserve_in as f64) + (amount_in as f64))
    }
}

/// Tensor-aware swap calculator (v0.2.0).
///
/// Extends SimpleSwapCalculator with:
/// - Dynamic fee adjustment based on pool coherence
/// - Slippage insurance for high-entropy swaps
/// - Price impact reduction for aligned trades
///
/// Based on REALMS Part IV: coherent pools have lower fees and better execution.
pub struct TensorSwapCalculator {
    base_swap_fee_bps: u64,
    base_protocol_fee_bps: u64,
    s_max: u64,
}

impl TensorSwapCalculator {
    pub fn new(swap_fee_bps: u64, protocol_fee_bps: u64, s_max: u64) -> Self {
        TensorSwapCalculator {
            base_swap_fee_bps: swap_fee_bps,
            base_protocol_fee_bps: protocol_fee_bps,
            s_max,
        }
    }

    /// Calculate dynamic fee based on pool coherence.
    /// Higher coherence = lower fees (better execution).
    pub fn calculate_dynamic_swap_fee(
        &self,
        amount: u64,
        pool_coherence: u64,
    ) -> u64 {
        // Coherence discount: 0 to 50% fee reduction at max coherence
        let coherence_ratio = (pool_coherence as f64) / (self.s_max as f64);
        let fee_discount = coherence_ratio * 0.5;
        let effective_fee_bps = ((self.base_swap_fee_bps as f64) * (1.0 - fee_discount)) as u64;

        (amount as u128 * effective_fee_bps as u128 / 10000) as u64
    }
}

impl SwapCalculator for TensorSwapCalculator {
    fn calculate_swap(
        &self,
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> SwapResult<SwapQuote> {
        // Use base simple calculator for now
        // In full implementation, would apply tensor adjustments
        let simple = SimpleSwapCalculator::new(self.base_swap_fee_bps, self.base_protocol_fee_bps);
        simple.calculate_swap(amount_in, reserve_in, reserve_out)
    }

    fn calculate_reverse_swap(
        &self,
        amount_out: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> SwapResult<u64> {
        let simple = SimpleSwapCalculator::new(self.base_swap_fee_bps, self.base_protocol_fee_bps);
        simple.calculate_reverse_swap(amount_out, reserve_in, reserve_out)
    }

    fn validate_slippage(&self, actual_out: u64, minimum_out: u64) -> SwapResult<()> {
        if actual_out < minimum_out {
            return Err(SwapError::SlippageExceeded);
        }
        Ok(())
    }

    fn calculate_lp_fees(&self, swap_amount: u64) -> u64 {
        let total_fee = (swap_amount as u128 * self.base_swap_fee_bps as u128 / 10000) as u64;
        let protocol_fee = (swap_amount as u128 * self.base_protocol_fee_bps as u128 / 10000) as u64;
        total_fee.saturating_sub(protocol_fee)
    }

    fn calculate_price_impact(
        &self,
        amount_in: u64,
        _amount_out: u64,
        reserve_in: u64,
        _reserve_out: u64,
    ) -> f64 {
        (amount_in as f64) / ((reserve_in as f64) + (amount_in as f64))
    }
}

/// Mock swap calculator for testing.
#[cfg(test)]
pub struct MockSwapCalculator {
    amount_out: u64,
    price_impact: f64,
}

#[cfg(test)]
impl MockSwapCalculator {
    pub fn new(amount_out: u64, price_impact: f64) -> Self {
        MockSwapCalculator {
            amount_out,
            price_impact,
        }
    }
}

#[cfg(test)]
impl SwapCalculator for MockSwapCalculator {
    fn calculate_swap(
        &self,
        _amount_in: u64,
        _reserve_in: u64,
        _reserve_out: u64,
    ) -> SwapResult<SwapQuote> {
        Ok(SwapQuote {
            amount_out: self.amount_out,
            swap_fee: 0,
            protocol_fee: 0,
            lp_fee: 0,
            price_impact: self.price_impact,
        })
    }

    fn calculate_reverse_swap(
        &self,
        _amount_out: u64,
        _reserve_in: u64,
        _reserve_out: u64,
    ) -> SwapResult<u64> {
        Ok(1000)
    }

    fn validate_slippage(&self, actual_out: u64, minimum_out: u64) -> SwapResult<()> {
        if actual_out < minimum_out {
            return Err(SwapError::SlippageExceeded);
        }
        Ok(())
    }

    fn calculate_lp_fees(&self, swap_amount: u64) -> u64 {
        (swap_amount as f64 * 0.0025) as u64
    }

    fn calculate_price_impact(
        &self,
        _amount_in: u64,
        _amount_out: u64,
        _reserve_in: u64,
        _reserve_out: u64,
    ) -> f64 {
        self.price_impact
    }
}
