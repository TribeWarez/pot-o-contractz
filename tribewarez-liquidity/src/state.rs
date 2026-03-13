use crate::errors::LiquidityError;
use anchor_lang::prelude::*;

#[account]
pub struct LiquidityPool {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub lp_token_mint: Pubkey,
    pub fee_bps: u16,
    pub admin: Pubkey,
    pub bump: u8,
}

impl LiquidityPool {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 32 + 2 + 32 + 1;

    pub fn calculate_swap_output(&self, amount_in: u64, from_token: Pubkey) -> Result<u64> {
        require!(amount_in > 0, LiquidityError::ZeroAmount);

        let (reserve_in, reserve_out) = if from_token == self.token_a {
            (self.reserve_a, self.reserve_b)
        } else if from_token == self.token_b {
            (self.reserve_b, self.reserve_a)
        } else {
            return Err(error!(LiquidityError::InvalidTokenPair));
        };

        require!(
            reserve_in > 0 && reserve_out > 0,
            LiquidityError::InsufficientLiquidity
        );

        let fee = amount_in
            .checked_mul(self.fee_bps as u64)
            .and_then(|v| v.checked_div(10000))
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;

        let amount_after_fee = amount_in
            .checked_sub(fee)
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;

        let numerator = amount_after_fee
            .checked_mul(reserve_out)
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;

        let denominator = reserve_in
            .checked_add(amount_after_fee)
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;

        let output = numerator
            .checked_div(denominator)
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;

        Ok(output)
    }

    pub fn calculate_lp_shares(&self, amount_a: u64, amount_b: u64) -> Result<u64> {
        if self.reserve_a == 0 && self.reserve_b == 0 {
            let geometric_mean = ((amount_a as u128)
                .checked_mul(amount_b as u128)
                .ok_or(error!(LiquidityError::ArithmeticOverflow))?)
            .isqrt();

            return Ok(geometric_mean as u64);
        }

        let shares_a = (amount_a as u128)
            .checked_mul(1000000u128)
            .and_then(|v| v.checked_div(self.reserve_a as u128))
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;

        let shares_b = (amount_b as u128)
            .checked_mul(1000000u128)
            .and_then(|v| v.checked_div(self.reserve_b as u128))
            .ok_or(error!(LiquidityError::ArithmeticOverflow))?;

        let shares = shares_a.min(shares_b);

        Ok(shares as u64)
    }

    pub fn calculate_withdraw_amounts(&self, shares: u64, total_shares: u64) -> Result<(u64, u64)> {
        require!(shares > 0 && total_shares > 0, LiquidityError::ZeroAmount);

        let amount_a = (shares as u128)
            .checked_mul(self.reserve_a as u128)
            .and_then(|v| v.checked_div(total_shares as u128))
            .ok_or(error!(LiquidityError::ArithmeticOverflow))? as u64;

        let amount_b = (shares as u128)
            .checked_mul(self.reserve_b as u128)
            .and_then(|v| v.checked_div(total_shares as u128))
            .ok_or(error!(LiquidityError::ArithmeticOverflow))? as u64;

        Ok((amount_a, amount_b))
    }

    pub fn update_reserves(&mut self, amount_a: u64, amount_b: u64, is_add: bool) -> Result<()> {
        if is_add {
            self.reserve_a = self
                .reserve_a
                .checked_add(amount_a)
                .ok_or(error!(LiquidityError::ArithmeticOverflow))?;
            self.reserve_b = self
                .reserve_b
                .checked_add(amount_b)
                .ok_or(error!(LiquidityError::ArithmeticOverflow))?;
        } else {
            self.reserve_a = self
                .reserve_a
                .checked_sub(amount_a)
                .ok_or(error!(LiquidityError::InsufficientLiquidity))?;
            self.reserve_b = self
                .reserve_b
                .checked_sub(amount_b)
                .ok_or(error!(LiquidityError::InsufficientLiquidity))?;
        }
        Ok(())
    }

    pub fn validate_pair(&self, token_a: Pubkey, token_b: Pubkey) -> Result<()> {
        if (self.token_a == token_a && self.token_b == token_b)
            || (self.token_a == token_b && self.token_b == token_a)
        {
            Ok(())
        } else {
            Err(error!(LiquidityError::InvalidTokenPair))
        }
    }
}

#[account]
pub struct PoolPosition {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub shares: u64,
    pub bump: u8,
}

impl PoolPosition {
    pub const LEN: usize = 32 + 32 + 8 + 1;
}

#[account]
pub struct PoolConfig {
    pub authority: Pubkey,
    pub default_fee_bps: u16,
    pub min_liquidity: u64,
    pub max_fee_bps: u16,
    pub bump: u8,
}

impl PoolConfig {
    pub const LEN: usize = 32 + 2 + 8 + 2 + 1;

    pub fn set_fee(&mut self, fee_bps: u16) -> Result<()> {
        require!(fee_bps <= self.max_fee_bps, LiquidityError::InvalidFee);
        self.default_fee_bps = fee_bps;
        Ok(())
    }
}

#[account]
pub struct PriceFeed {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub price_a_to_b: u64,
    pub last_update: i64,
    pub bump: u8,
}

impl PriceFeed {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 1;

    pub fn update_price(&mut self, new_price: u64) -> Result<()> {
        self.price_a_to_b = new_price;
        self.last_update = 1; // Simplified without sysvar
        Ok(())
    }

    pub fn calculate_twap(&self, old_price: u64) -> Result<u64> {
        if self.last_update == 0 {
            return Ok(self.price_a_to_b);
        }
        let twap = old_price.saturating_add(self.price_a_to_b) / 2;
        Ok(twap)
    }
}
