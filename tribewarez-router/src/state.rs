use crate::errors::RouterError;
use anchor_lang::prelude::*;

#[account]
pub struct SwapRoute {
    pub from_token: Pubkey,
    pub to_token: Pubkey,
    pub via_token: Option<Pubkey>,
    pub fee_bps: u16,
    pub liquidity: u64,
    pub bump: u8,
}

impl SwapRoute {
    pub const LEN: usize = 32 + 32 + 33 + 2 + 8 + 1;

    pub fn calculate_output(&self, amount_in: u64) -> Result<u64> {
        let fee = amount_in
            .checked_mul(self.fee_bps as u64)
            .and_then(|v| v.checked_div(10000))
            .ok_or(error!(RouterError::ArithmeticOverflow))?;

        let amount_after_fee = amount_in
            .checked_sub(fee)
            .ok_or(error!(RouterError::ArithmeticOverflow))?;

        let output = amount_after_fee
            .checked_mul(self.liquidity)
            .and_then(|v| v.checked_div(amount_in.saturating_add(1)))
            .ok_or(error!(RouterError::ArithmeticOverflow))?;

        Ok(output)
    }

    pub fn validate_path(&self, expected_from: Pubkey, expected_to: Pubkey) -> Result<()> {
        if self.from_token != expected_from || self.to_token != expected_to {
            Err(error!(RouterError::InvalidSwapPath))
        } else {
            Ok(())
        }
    }

    pub fn has_intermediate_token(&self) -> bool {
        self.via_token.is_some()
    }

    pub fn get_path_length(&self) -> usize {
        if self.via_token.is_some() {
            3
        } else {
            2
        }
    }
}

#[account]
pub struct RouteConfig {
    pub authority: Pubkey,
    pub default_fee_bps: u16,
    pub max_hops: u8,
    pub enabled: bool,
    pub bump: u8,
}

impl RouteConfig {
    pub const LEN: usize = 32 + 2 + 1 + 1 + 1;

    pub fn set_fee(&mut self, fee_bps: u16) -> Result<()> {
        self.default_fee_bps = fee_bps;
        Ok(())
    }

    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[account]
pub struct SwapQuote {
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub path: Vec<Pubkey>,
    pub price_impact: u64,
    pub timestamp: i64,
}

impl SwapQuote {
    pub fn calculate_price_impact(&self, amount_in: u64, old_price: u64, new_price: u64) -> u64 {
        if old_price == 0 {
            return 0;
        }
        let impact = ((old_price - new_price) * 10000) / old_price;
        impact
    }

    pub fn validate_slippage(&self, min_output: u64) -> Result<()> {
        if self.amount_out < min_output {
            Err(error!(RouterError::SlippageExceeded))
        } else {
            Ok(())
        }
    }
}
