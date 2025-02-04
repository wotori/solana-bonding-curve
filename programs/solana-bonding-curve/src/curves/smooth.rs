use anchor_lang::prelude::*;

use crate::errors::CustomError;

//==============================================================================
/// BondingCurveTrait defines the core bonding curve functions.
pub trait BondingCurveTrait {
    /// Buys with exact base_tokens in, returning the exact number of minted tokens (Δy).
    fn buy_exact_input(&mut self, base_in: u64) -> std::result::Result<u64, CustomError>;

    /// Buys an exact number of tokens out (tokens_out), returning the exact base_tokens required.
    fn buy_exact_output(&mut self, tokens_out: u64) -> std::result::Result<u64, CustomError>;

    /// Sells an exact number of tokens in, returning the exact base_tokens out.
    fn sell_exact_input(&mut self, tokens_in: u64) -> std::result::Result<u64, CustomError>;

    /// Sells enough tokens to receive exactly `base_out` from the curve.
    /// Returns the number of "pool tokens" that must be burned.
    fn sell_exact_output(&mut self, base_out: u64) -> std::result::Result<u64, CustomError>;
}

//==============================================================================
/// A smooth bonding curve tracking the total base asset (e.g., SOL, XBT) deposited.
///
/// Formula: y(x) = A - (K / (C + x))
/// - A = asymptotic max token supply (in integer "token units")
/// - K = (token * lamport), controlling how quickly we approach A
/// - C = virtual pool offset (in base_tokens)
/// - x = total base asset deposited so far (in base_tokens)
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SmoothBondingCurve {
    /// Asymptotic total token supply (in "raw" tokens)
    pub a_total_tokens: u64,
    /// Controls how quickly we approach A (token * lamport)
    pub k_virtual_pool_offset: u128,
    /// Virtual pool offset (in base_tokens)
    pub c_bonding_scale_factor: u64,
    /// Total base asset deposited (in base_tokens)
    pub x_total_base_deposit: u64,
}

impl SmoothBondingCurve {
    /// Calculates the total minted tokens at `x_val` base_tokens in the pool:
    /// y(x) = A - (K / (C + x)) (all integer math)
    fn y_of_x(&self, x_val: u64) -> u64 {
        let denom = self
            .c_bonding_scale_factor
            .checked_add(x_val)
            .unwrap_or(u64::MAX); // If overflow occurs, fallback to maximum value.
        let k_over_denom = self
            .k_virtual_pool_offset
            .checked_div(denom as u128)
            .unwrap_or(0);
        self.a_total_tokens.saturating_sub(k_over_denom as u64)
    }

    /// Computes the new x (x') for a target y = new_y.
    ///
    /// Rearranging the formula: new_y = A - (K / (C + x')):
    ///    A - new_y = K / (C + x')
    /// => (C + x') = K / (A - new_y)
    /// => x' = (K / (A - new_y)) - C
    ///
    /// Returns an error if new_y ≥ A or if computations result in an arithmetic error.
    fn solve_for_x_prime(&self, new_y: u128) -> std::result::Result<u128, CustomError> {
        if new_y >= self.a_total_tokens as u128 {
            return Err(CustomError::InsufficientTokenSupply);
        }
        let a_minus_new_y = (self.a_total_tokens as u128)
            .checked_sub(new_y)
            .ok_or(CustomError::InsufficientTokenSupply)?;

        let big_val = self
            .k_virtual_pool_offset
            .checked_div(a_minus_new_y)
            .ok_or(CustomError::MathOverflow)?;

        if big_val < self.c_bonding_scale_factor as u128 {
            return Err(CustomError::MathOverflow);
        }

        big_val
            .checked_sub(self.c_bonding_scale_factor as u128)
            .ok_or(CustomError::MathOverflow)
    }
}

impl BondingCurveTrait for SmoothBondingCurve {
    /// Buys with exact base_tokens in, returning the exact number of minted tokens (Δy).
    fn buy_exact_input(&mut self, base_in: u64) -> std::result::Result<u64, CustomError> {
        let old_y = self.y_of_x(self.x_total_base_deposit);
        let new_x = self
            .x_total_base_deposit
            .checked_add(base_in)
            .ok_or(CustomError::MathOverflow)?;
        let new_y = self.y_of_x(new_x);
        let minted = new_y.checked_sub(old_y).ok_or(CustomError::MathOverflow)?;
        self.x_total_base_deposit = new_x;
        Ok(minted)
    }

    /// Buys an exact number of tokens out (tokens_out), returning the exact base_tokens required.
    fn buy_exact_output(&mut self, tokens_out: u64) -> std::result::Result<u64, CustomError> {
        let old_y = self.y_of_x(self.x_total_base_deposit);
        let new_y = old_y
            .checked_add(tokens_out)
            .ok_or(CustomError::MathOverflow)?;
        let x_prime = self.solve_for_x_prime(new_y as u128)?;
        let base_in = x_prime
            .checked_sub(self.x_total_base_deposit as u128)
            .ok_or(CustomError::MathOverflow)?;
        self.x_total_base_deposit = x_prime as u64;
        Ok(base_in as u64)
    }

    /// Sells an exact number of tokens in, returning the exact base_tokens out.
    fn sell_exact_input(&mut self, tokens_in: u64) -> std::result::Result<u64, CustomError> {
        let old_y = self.y_of_x(self.x_total_base_deposit);
        let new_y = old_y
            .checked_sub(tokens_in)
            .ok_or(CustomError::InsufficientTokenSupply)?;
        let x_prime = self.solve_for_x_prime(new_y as u128)?;
        let base_out = (self.x_total_base_deposit as u128)
            .checked_sub(x_prime)
            .ok_or(CustomError::MathOverflow)?;
        self.x_total_base_deposit = x_prime as u64;
        Ok(base_out as u64)
    }

    /// Sells enough tokens to receive exactly `base_out` from the curve.
    /// Returns the number of "pool tokens" that must be burned.
    fn sell_exact_output(&mut self, base_out: u64) -> std::result::Result<u64, CustomError> {
        let old_x = self.x_total_base_deposit;
        let old_y = self.y_of_x(old_x);

        if (base_out as u128) > (old_x as u128) {
            return Err(CustomError::InsufficientTokenSupply);
        }

        let new_x = (old_x as u128)
            .checked_sub(base_out as u128)
            .ok_or(CustomError::MathOverflow)?;
        let new_y = self.y_of_x(new_x as u64);
        let tokens_to_burn = old_y.checked_sub(new_y).ok_or(CustomError::MathOverflow)?;
        self.x_total_base_deposit = new_x as u64;
        Ok(tokens_to_burn as u64)
    }
}

//==============================================================================
// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

    mod xyber_params {
        // Example values; replace with actual parameters
        pub const _TOTAL_TOKENS: u64 = 100_000_000;
        pub const _BONDING_SCALE_FACTOR: u128 = 1_000_000_000_000;
        pub const _VIRTUAL_POOL_OFFSET: u64 = 30_000_000_000;
    }

    #[test]
    fn test_buy_exact_input() {
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };

        let base_in = (0.001 * LAMPORTS_PER_SOL as f64) as u64;
        let minted = curve.buy_exact_input(base_in).unwrap();
        println!("minted: {}", minted);

        assert!(
            (34_600..36_700).contains(&minted),
            "Minted tokens out of expected range: {}",
            minted
        );
    }

    #[test]
    fn test_buy_exact_output() {
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };

        let tokens_out = 10_000;
        println!("Initial curve state: {:?}", curve);
        println!("Tokens to buy: {}", tokens_out);

        let lamports_required = curve.buy_exact_output(tokens_out).unwrap();

        println!("Lamports required: {}", lamports_required);
        println!("Curve state after buy: {:?}", curve);

        assert!(
            lamports_required > 0,
            "Lamports required should be greater than 0"
        );

        let new_y = curve.y_of_x(curve.x_total_base_deposit);
        println!("New y after buy: {}", new_y);

        assert_eq!(
            new_y, tokens_out,
            "The curve state should reflect the exact number of tokens bought"
        );

        println!(
            "To buy {} tokens, {} base_tokens are required",
            tokens_out, lamports_required
        );
    }

    #[test]
    fn test_buy_various_inputs() {
        let fractions = [10.0, 1.0, 0.1, 0.01, 0.001, 0.0001];
        let mut prev_minted = u128::MAX;

        for fraction in fractions {
            let mut curve = SmoothBondingCurve {
                a_total_tokens: xyber_params::_TOTAL_TOKENS,
                k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
                c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
                x_total_base_deposit: 0,
            };

            let lamports_in = (LAMPORTS_PER_SOL as f64 * fraction) as u64;
            let minted = curve.buy_exact_input(lamports_in).unwrap();

            println!(
                "fraction = {:.4}, lamports_in = {}, minted = {}",
                fraction, lamports_in, minted
            );

            assert!(
                minted > 0,
                "Expected a positive number of tokens for fraction={}",
                fraction
            );

            // Ensure monotonic decrease in minted tokens as fraction decreases
            assert!(
                (minted as u128) <= prev_minted,
                "Expected minted tokens to decrease as fraction decreases"
            );
            prev_minted = minted as u128;
        }
    }

    #[test]
    fn test_sell_exact_input() {
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };

        let sol_in = (0.1 * LAMPORTS_PER_SOL as f64) as u64;
        let minted_tokens = curve.buy_exact_input(sol_in).unwrap();

        let tokens_to_sell = minted_tokens / 2;
        println!("tokens_to_sell: {}", tokens_to_sell);

        let lamports_out = curve.sell_exact_input(tokens_to_sell).unwrap();
        println!("lamports_out: {}", lamports_out);

        assert!(
            lamports_out > 0,
            "Should receive some base_tokens when selling tokens"
        );
    }

    #[test]
    fn test_sell_exact_output() {
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };

        let base_in = (0.1 * LAMPORTS_PER_SOL as f64) as u64;
        let minted_tokens = curve.buy_exact_input(base_in).unwrap();
        assert!(minted_tokens > 0, "Initial token minting failed");

        let base_out = curve.x_total_base_deposit / 2;

        let tokens_burned = curve.sell_exact_output(base_out).unwrap();

        let remaining_lamports_in_pool = curve.x_total_base_deposit;
        let expected_after_withdraw = base_in
            .checked_sub(base_out)
            .expect("Subtraction error in test_sell_exact_output");
        assert_eq!(
            remaining_lamports_in_pool, expected_after_withdraw,
            "Pool's base_tokens did not decrease correctly by base_out"
        );

        let remaining_tokens = curve.y_of_x(curve.x_total_base_deposit);
        let real_burn = minted_tokens.saturating_sub(remaining_tokens);
        assert_eq!(
            tokens_burned, real_burn,
            "Mismatch in token burn calculation"
        );

        println!(
            "Requested {} base tokens, curve granted that amount. \
             We had to burn {} tokens (by formula). \
             Now {} minted tokens remain in the curve (y_of_x)",
            base_out, tokens_burned, remaining_tokens
        );
    }

    #[test]
    fn test_buy_sell_symmetry() {
        // (A) Buy Exact Input -> Sell Exact Input
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };
        let lamports_in_a: u64 = 2 * LAMPORTS_PER_SOL; // Purchasing with 2 SOL
        let minted_a = curve.buy_exact_input(lamports_in_a).unwrap();
        println!(
            "(A) Bought {} tokens for {} base_tokens",
            minted_a, lamports_in_a
        );

        let lamports_out_a = curve.sell_exact_input(minted_a).unwrap();
        println!(
            "(A) Sold back {} tokens, got {} base_tokens",
            minted_a, lamports_out_a
        );

        let tolerance_a = 0;
        let diff_a = lamports_out_a as i64 - lamports_in_a as i64;
        assert!(
            diff_a.abs() <= tolerance_a,
            "Unexpected slippage in (A): diff={} (got {}, expected {})",
            diff_a,
            lamports_out_a,
            lamports_in_a
        );

        // (B) Buy Exact Output -> Sell Exact Input
        let mut curve2 = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };
        let tokens_out_b = 50_000;
        let lamports_in_b = curve2.buy_exact_output(tokens_out_b).unwrap();
        println!(
            "(B) Bought {} tokens (exact output) for {} base_tokens",
            tokens_out_b, lamports_in_b
        );

        let lamports_out_b = curve2.sell_exact_input(tokens_out_b).unwrap();
        println!(
            "(B) Sold {} tokens, got back {} base_tokens",
            tokens_out_b, lamports_out_b
        );

        let tolerance_b = 0;
        let diff_b = lamports_out_b as i64 - lamports_in_b as i64;
        assert!(
            diff_b.abs() <= tolerance_b,
            "Unexpected slippage in (B): diff={} (got {}, expected {})",
            diff_b,
            lamports_out_b,
            lamports_in_b
        );
    }

    /// Estimates the marginal price (dX/dY) in float, used for logging/tracing price growth.
    fn approximate_price(curve: &SmoothBondingCurve) -> f64 {
        let denom = (curve.c_bonding_scale_factor as f64) + (curve.x_total_base_deposit as f64);
        let k = curve.k_virtual_pool_offset as f64;
        // For y(x) = A - (K / (C + x)), the local slope dX/dY is:
        //     dX/dY = (C + x)^2 / K
        (denom * denom) / k
    }

    #[test]
    fn test_buy_until_70k_liquidity() {
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };

        let target_liquidity_usd = 70_000.0;
        let sol_price_usd = 250.0;
        let target_sol_in_pool = target_liquidity_usd / sol_price_usd;
        let base_in_per_step: u64 = LAMPORTS_PER_SOL; // 1 SOL per iteration

        let max_iterations: u16 = 1000;
        let mut iteration = 0;

        while (curve.x_total_base_deposit as f64) / (LAMPORTS_PER_SOL as f64) < target_sol_in_pool {
            iteration += 1;
            if iteration > max_iterations {
                panic!("Exceeded max iterations; something might be off.");
            }

            let minted = curve.buy_exact_input(base_in_per_step).unwrap();
            let total_pool_sol = (curve.x_total_base_deposit as f64) / (LAMPORTS_PER_SOL as f64);
            let new_price = approximate_price(&curve);

            println!(
                "Iteration {}: +1 SOL => minted {} tokens, approx price={:.2e}, total SOL={:.4}",
                iteration, minted, new_price, total_pool_sol
            );
        }

        let final_sol = (curve.x_total_base_deposit as f64) / (LAMPORTS_PER_SOL as f64);
        let final_usd = final_sol * sol_price_usd;
        println!(
            "Reached target liquidity.\n  - Final SOL in pool: {:.2}\n  - Final USD value: ${:.2}\n",
            final_sol, final_usd
        );

        assert!(
            final_usd >= 70_000.0,
            "Expected at least $70k in the pool, but got ${:.2}",
            final_usd
        );
    }
}
