use super::traits::BondingCurveTrait;
use anchor_lang::prelude::*;

/// A smooth bonding curve tracking the total base asset (e.g., SOL, XBT) deposited.
///
/// Formula: y(x) = A - K / (C + x)
/// - A = asymptotic max token supply (in integer "token units")
/// - K = dimension is (token * lamport), controlling how quickly we approach A
/// - C = virtual pool offset (in base_tokens), e.g., 30 SOL -> 30_000_000_000 base_tokens
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
    /// y(x) = A - (K / (C + x))   (all integer math)
    fn y_of_x(&self, x_val: u64) -> u64 {
        let denom = (self.c_bonding_scale_factor).saturating_add(x_val);
        let k_over_denom = self.k_virtual_pool_offset.saturating_div(denom as u128);
        (self.a_total_tokens).saturating_sub(k_over_denom as u64)
    }

    /// Computes the `x` (new_x) for a target y = new_y.
    ///
    /// Re-arranges the formula: new_y = A - (K / (C + x')):
    ///    A - new_y = K / (C + x')
    /// => (C + x') = K / (A - new_y)
    /// => x' = (K / (A - new_y)) - C
    ///
    /// *Panics* if new_y >= A or if (C + x') < C (underflow).
    fn solve_for_x_prime(&self, new_y: u128) -> u128 {
        if new_y >= self.a_total_tokens as u128 {
            panic!("Requested new_y exceeds or equals the asymptote A");
        }
        let a_minus_new_y = (self.a_total_tokens as u128)
            .checked_sub(new_y)
            .expect("new_y too large, exceeds curve's max supply");

        let big_val = self
            .k_virtual_pool_offset
            .checked_div(a_minus_new_y)
            .expect("Division by zero or K too small");

        if big_val < self.c_bonding_scale_factor as u128 {
            panic!("Curve underflow: (C + x') < C");
        }

        big_val
            .checked_sub(self.c_bonding_scale_factor as u128)
            .expect("Internal underflow computing x'")
    }
}

impl BondingCurveTrait for SmoothBondingCurve {
    /// Buys with exact base_tokens in, returning the exact number of minted tokens (Δy).
    fn buy_exact_input(&mut self, base_in: u64) -> u64 {
        let old_y = self.y_of_x(self.x_total_base_deposit);
        let new_x = self.x_total_base_deposit.saturating_add(base_in);
        let new_y = self.y_of_x(new_x);
        let minted = new_y.saturating_sub(old_y);

        self.x_total_base_deposit = new_x;
        minted as u64
    }

    /// Buys an exact number of tokens out (tokens_out), returning the exact base_tokens required.
    fn buy_exact_output(&mut self, tokens_out: u64) -> u64 {
        let old_y = self.y_of_x(self.x_total_base_deposit);
        let new_y = old_y
            .checked_add(tokens_out)
            .expect("Cannot buy a negative amount or overflow tokens");

        let x_prime = self.solve_for_x_prime(new_y as u128);
        let base_in = x_prime
            .checked_sub(self.x_total_base_deposit as u128)
            .expect("Not enough base_tokens to buy these tokens");

        self.x_total_base_deposit = x_prime as u64;
        base_in as u64
    }

    /// Sells an exact number of tokens in, returning the exact base_tokens out.
    fn sell_exact_input(&mut self, tokens_in: u64) -> u64 {
        let old_y = self.y_of_x(self.x_total_base_deposit);
        let new_y = old_y
            .checked_sub(tokens_in)
            .expect("Cannot sell more tokens than curve state holds");

        let x_prime = self.solve_for_x_prime(new_y as u128);
        let base_out = (self.x_total_base_deposit as u128)
            .checked_sub(x_prime)
            .expect("logic error: x' is larger than old_x");

        self.x_total_base_deposit = x_prime as u64;
        base_out as u64
    }

    /// Sells enough tokens to receive exactly `base_out` from the curve.
    /// Returns the number of "pool tokens" that must be burned (`tokens_in`).
    fn sell_exact_output(&mut self, base_out: u64) -> u64 {
        let old_x = self.x_total_base_deposit;
        let old_y = self.y_of_x(old_x);

        if base_out as u128 > old_x as u128 {
            panic!("Not enough base_tokens in the pool to withdraw this amount");
        }

        let new_x = old_x as u128 - base_out as u128;
        let new_y = self.y_of_x(new_x as u64);

        let tokens_to_burn = old_y.saturating_sub(new_y);
        self.x_total_base_deposit = new_x as u64;
        tokens_to_burn as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xyber_params;
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

    #[test]
    fn test_buy_exact_input() {
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };

        let base_in = (0.001 * LAMPORTS_PER_SOL as f64) as u64;
        let minted = curve.buy_exact_input(base_in);
        println!("minted: {}", minted);

        assert!(
            (34_600..36_700).contains(&(minted as u64)),
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

        let lamports_required = curve.buy_exact_output(tokens_out);

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
            let minted = curve.buy_exact_input(lamports_in) as u64;

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
                minted <= prev_minted as u64,
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
        let minted_tokens = curve.buy_exact_input(sol_in);

        let tokens_to_sell = minted_tokens / 2;
        println!("tokens_to_sell: {}", tokens_to_sell);

        let lamports_out = curve.sell_exact_input(tokens_to_sell);
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
        let minted_tokens = curve.buy_exact_input(base_in);
        assert!(minted_tokens > 0, "Initial token minting failed");

        let base_out = curve.x_total_base_deposit / 2;

        let tokens_burned = curve.sell_exact_output(base_out);

        let remaining_lamports_in_pool = curve.x_total_base_deposit;
        let expected_after_withdraw = base_in - base_out;
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
            "Requested {} base tokens, curve granted that amount. 
             We had to burn {} tokens (by formula). 
             Now {} minted tokens remain in the curve (y_of_x)",
            base_out, tokens_burned, remaining_tokens
        );
    }

    #[test]
    fn test_buy_sell_symmetry() {
        use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

        // (A) Buy Exact Input -> Sell Exact Input
        let mut curve = SmoothBondingCurve {
            a_total_tokens: xyber_params::_TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::_BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::_VIRTUAL_POOL_OFFSET,
            x_total_base_deposit: 0,
        };
        let lamports_in_a: u64 = 2 * LAMPORTS_PER_SOL; // Purchasing with 2 SOL
        let minted_a = curve.buy_exact_input(lamports_in_a);
        println!(
            "(A) Bought {} tokens for {} base_tokens",
            minted_a, lamports_in_a
        );

        let lamports_out_a = curve.sell_exact_input(minted_a);
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
        let lamports_in_b = curve2.buy_exact_output(tokens_out_b);
        println!(
            "(B) Bought {} tokens (exact output) for {} base_tokens",
            tokens_out_b, lamports_in_b
        );

        let lamports_out_b = curve2.sell_exact_input(tokens_out_b);
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
        let base_in_per_step: u64 = LAMPORTS_PER_SOL; // 1 SOL each iteration

        let max_iterations: u16 = 1000;
        let mut iteration = 0;

        while (curve.x_total_base_deposit as f64) / (LAMPORTS_PER_SOL as f64) < target_sol_in_pool {
            iteration += 1;
            if iteration > max_iterations {
                panic!("Exceeded max iterations; something might be off.");
            }

            let minted = curve.buy_exact_input(base_in_per_step);
            let total_pool_sol = (curve.x_total_base_deposit as f64) / (LAMPORTS_PER_SOL as f64);
            let new_price = approximate_price(&curve);

            println!(
                "Iteration {iteration}: +1 SOL => minted {minted} tokens, \
                 approx price={:.2e}, total SOL={:.4}",
                new_price, total_pool_sol
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
