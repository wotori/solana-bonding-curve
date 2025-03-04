use anchor_lang::prelude::*;

use crate::errors::CustomError;

//==============================================================================
/// BondingCurveTrait defines the core bonding curve functions.
///
/// Here, each method accepts:
/// - `old_x`: the current base_tokens in the pool (before this new operation),
/// - and either `base_in`, `tokens_out`, etc.,
/// returning either the minted/burned tokens or the base tokens
/// along with the updated pool balance `new_x` if needed.
///
/// This way, the curve does NOT store `x` internally.
pub trait BondingCurveTrait {
    /// Buys with exact base_tokens in, returning the exact number of minted tokens (Δy)
    /// plus the new x in the pool.
    fn buy_exact_input(
        &self,
        old_x: u64,
        base_in: u64,
    ) -> std::result::Result<(u64, u64), CustomError>;

    /// Buys an exact number of tokens out (tokens_out), returning the exact base_tokens required,
    /// plus the new x in the pool.
    fn buy_exact_output(
        &self,
        old_x: u64,
        tokens_out: u64,
    ) -> std::result::Result<(u64, u64), CustomError>;

    /// Sells an exact number of tokens in, returning the exact base_tokens out,
    /// plus the new x in the pool.
    fn sell_exact_input(
        &self,
        old_x: u64,
        tokens_in: u64,
    ) -> std::result::Result<(u64, u64), CustomError>;

    /// Sells enough tokens to receive exactly `base_out` from the curve.
    /// Returns the number of "pool tokens" that must be burned,
    /// plus the new x in the pool.
    fn sell_exact_output(
        &self,
        old_x: u64,
        base_out: u64,
    ) -> std::result::Result<(u64, u64), CustomError>;
}

//==============================================================================
/// A smooth bonding curve referencing the base asset (e.g., SOL, XBT) deposited.
///
/// Formula: y(x) = A - (K / (C + x))
/// - A = asymptotic max token supply (in integer "token units")
/// - K = (token * lamport), controlling how quickly we approach A
/// - C = virtual pool offset (in base_tokens)
///
/// NOTE: We no longer store `x_total_base_deposit` inside the struct.
/// Instead, the caller passes the current x each time.
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SmoothBondingCurve {
    /// Asymptotic total token supply (in "raw" tokens)
    pub a_total_tokens: u64,
    /// Controls how quickly we approach A (token * lamport)
    pub k_virtual_pool_offset: u128,
    /// Virtual pool offset (in base_tokens)
    pub c_bonding_scale_factor: u64,
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
    /// Buys with exact base_tokens in, returning the exact number of minted tokens (Δy),
    /// plus the updated x.
    fn buy_exact_input(
        &self,
        old_x: u64,
        base_in: u64,
    ) -> std::result::Result<(u64, u64), CustomError> {
        // new_x = old_x + base_in
        let new_x = old_x
            .checked_add(base_in)
            .ok_or(CustomError::MathOverflow)?;

        let old_y = self.y_of_x(old_x);
        let new_y = self.y_of_x(new_x);

        let minted = new_y.checked_sub(old_y).ok_or(CustomError::MathOverflow)?;

        Ok((minted, new_x))
    }

    /// Buys an exact number of tokens out (tokens_out), returning the exact base_tokens required,
    /// plus the updated x.
    fn buy_exact_output(
        &self,
        old_x: u64,
        tokens_out: u64,
    ) -> std::result::Result<(u64, u64), CustomError> {
        // old_y = y_of_x(old_x)
        let old_y = self.y_of_x(old_x);

        // new_y = old_y + tokens_out
        let new_y = old_y
            .checked_add(tokens_out)
            .ok_or(CustomError::MathOverflow)?;

        // x' = solve_for_x_prime(new_y)
        let x_prime = self.solve_for_x_prime(new_y as u128)?;

        // base_in = x_prime - old_x
        let base_in = x_prime
            .checked_sub(old_x as u128)
            .ok_or(CustomError::MathOverflow)?;

        Ok((base_in as u64, x_prime as u64))
    }

    /// Sells an exact number of tokens in, returning the exact base_tokens out,
    /// plus the updated x.
    fn sell_exact_input(
        &self,
        old_x: u64,
        tokens_in: u64,
    ) -> std::result::Result<(u64, u64), CustomError> {
        let old_y = self.y_of_x(old_x);

        // new_y = old_y - tokens_in
        let new_y = old_y
            .checked_sub(tokens_in)
            .ok_or(CustomError::InsufficientTokenSupply)?;

        // x_prime = solve_for_x_prime(new_y)
        let x_prime = self.solve_for_x_prime(new_y as u128)?;

        // base_out = old_x - x_prime
        let base_out = (old_x as u128)
            .checked_sub(x_prime)
            .ok_or(CustomError::MathOverflow)?;

        Ok((base_out as u64, x_prime as u64))
    }

    /// Sells enough tokens to receive exactly `base_out` from the curve.
    /// Returns the number of "pool tokens" that must be burned,
    /// plus the updated x.
    fn sell_exact_output(
        &self,
        old_x: u64,
        base_out: u64,
    ) -> std::result::Result<(u64, u64), CustomError> {
        let old_y = self.y_of_x(old_x);

        if (base_out as u128) > (old_x as u128) {
            return Err(CustomError::InsufficientTokenSupply);
        }

        // new_x = old_x - base_out
        let new_x = (old_x as u128)
            .checked_sub(base_out as u128)
            .ok_or(CustomError::MathOverflow)?;

        // new_y = y_of_x(new_x)
        let new_y = self.y_of_x(new_x as u64);

        // tokens_to_burn = old_y - new_y
        let tokens_to_burn = old_y.checked_sub(new_y).ok_or(CustomError::MathOverflow)?;

        Ok((tokens_to_burn as u64, new_x as u64))
    }
}

//==============================================================================
// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

    mod xyber_params {
        use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

        // Example values; replace with actual parameters
        pub const TOTAL_TOKENS: u64 = 1_073_000_191;
        pub const BONDING_SCALE_FACTOR: u128 = 32_190_005_730 * (LAMPORTS_PER_SOL as u128);
        pub const VIRTUAL_POOL_OFFSET: u64 = 30 * LAMPORTS_PER_SOL;
    }

    /// Helper function for building the default test curve.
    fn default_curve() -> SmoothBondingCurve {
        SmoothBondingCurve {
            a_total_tokens: xyber_params::TOTAL_TOKENS,
            k_virtual_pool_offset: xyber_params::BONDING_SCALE_FACTOR,
            c_bonding_scale_factor: xyber_params::VIRTUAL_POOL_OFFSET,
        }
    }

    #[test]
    fn test_buy_exact_input() {
        let curve = default_curve();
        let old_x = 0; // start with empty pool
        let base_in = (10 * LAMPORTS_PER_SOL) as u64;

        let (minted, new_x) = curve.buy_exact_input(old_x, base_in).unwrap();
        println!("minted: {}", minted);
        println!("new_x (pool deposit) after buy: {}", new_x);

        // We'll do a rough check on minted tokens
        assert!(
            (265_250_000..270_300_000).contains(&minted),
            "Minted tokens out of expected range: {}",
            minted
        );
    }

    #[test]
    fn test_buy_exact_output() {
        let curve = default_curve();

        // Suppose old_x is 0 at the beginning.
        let old_x = 0;
        let tokens_out = 10_000;

        let (lamports_required, new_x) = curve.buy_exact_output(old_x, tokens_out).unwrap();
        println!("Tokens to buy: {}", tokens_out);
        println!("Lamports required: {}", lamports_required);
        println!("New x (pool deposit) after buy: {}", new_x);

        assert!(
            lamports_required > 0,
            "Lamports required should be greater than 0"
        );

        // Let's confirm y(new_x) == tokens_out
        let real_new_y = curve.y_of_x(new_x);
        assert_eq!(
            real_new_y, tokens_out,
            "The curve state should reflect the exact number of tokens bought"
        );
    }

    #[test]
    fn test_buy_various_inputs() {
        let fractions = [10.0, 1.0, 0.1, 0.01, 0.001, 0.0001];
        let mut prev_minted = u128::MAX;
        let curve = default_curve();

        for fraction in fractions {
            // each iteration starts a new test scenario with old_x=0
            let old_x_local = 0;

            let lamports_in = (LAMPORTS_PER_SOL as f64 * fraction) as u64;
            let (minted, new_x) = curve.buy_exact_input(old_x_local, lamports_in).unwrap();

            println!(
                "fraction = {:.4}, lamports_in = {}, minted = {}, new_x={}",
                fraction, lamports_in, minted, new_x
            );

            assert!(
                minted > 0,
                "Expected a positive number of tokens for fraction={}",
                fraction
            );

            // Ensure minted tokens decrease as fraction decreases (roughly)
            assert!(
                (minted as u128) <= prev_minted,
                "Expected minted tokens to decrease as fraction decreases"
            );
            prev_minted = minted as u128;

            // For clarity, we do not carry new_x over to the next fraction test.
            // Each test fraction is from an empty pool scenario.
        }
    }

    #[test]
    fn test_sell_exact_input() {
        let curve = default_curve();
        let mut x = 0;

        // First, buy some tokens so we can sell them.
        let sol_in = (0.1 * LAMPORTS_PER_SOL as f64) as u64;
        let (minted_tokens, new_x) = curve.buy_exact_input(x, sol_in).unwrap();
        x = new_x; // update the pool deposit
        assert!(minted_tokens > 0);

        // Sell half the tokens
        let tokens_to_sell = minted_tokens / 2;
        let (lamports_out, next_x) = curve.sell_exact_input(x, tokens_to_sell).unwrap();
        x = next_x;
        println!("lamports_out: {}", lamports_out);
        println!("new pool x after sell: {}", x);

        assert!(
            lamports_out > 0,
            "Should receive some base_tokens when selling tokens"
        );
    }

    #[test]
    fn test_sell_exact_output() {
        let curve = default_curve();
        let mut x = 0;

        // Buy some tokens first
        let base_in = (0.1 * LAMPORTS_PER_SOL as f64) as u64;
        let (minted_tokens, new_x) = curve.buy_exact_input(x, base_in).unwrap();
        x = new_x;
        assert!(minted_tokens > 0, "Initial token minting failed");

        // Let's request exactly half of the current base_in pool
        let base_out = x / 2;

        let (tokens_burned, after_x) = curve.sell_exact_output(x, base_out).unwrap();
        println!("tokens_burned: {}", tokens_burned);
        println!("new pool x after sell: {}", after_x);

        // Check that the new x is as expected
        let expected_after_withdraw = x.checked_sub(base_out).unwrap();
        assert_eq!(
            after_x, expected_after_withdraw,
            "Pool's base_tokens did not decrease correctly by base_out"
        );

        // Check the real burn
        let old_y = curve.y_of_x(x);
        let new_y = curve.y_of_x(after_x);
        let real_burn = old_y.saturating_sub(new_y);
        assert_eq!(
            tokens_burned, real_burn,
            "Mismatch in token burn calculation"
        );
    }

    #[test]
    fn test_buy_sell_symmetry() {
        // (A) Buy Exact Input -> Sell Exact Input (from scratch)
        let curve = default_curve();

        // Start with an empty pool
        let mut x = 0;
        let lamports_in_a: u64 = 2 * LAMPORTS_PER_SOL; // Purchasing with 2 SOL

        let (minted_a, x_after_buy) = curve.buy_exact_input(x, lamports_in_a).unwrap();
        x = x_after_buy;
        println!(
            "(A) Bought {} tokens for {} base_tokens, new x={}",
            minted_a, lamports_in_a, x
        );

        let (lamports_out_a, x_after_sell) = curve.sell_exact_input(x, minted_a).unwrap();
        x = x_after_sell;
        println!(
            "(A) Sold back {} tokens, got {} base_tokens, new x={}",
            minted_a, lamports_out_a, x
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
        // We'll reset the pool to empty
        let mut x2 = 0;
        let tokens_out_b = 50_000;
        let (lamports_in_b, x2_after_buy) = curve.buy_exact_output(x2, tokens_out_b).unwrap();
        x2 = x2_after_buy;
        println!(
            "(B) Bought {} tokens (exact output) for {} base_tokens, new x={}",
            tokens_out_b, lamports_in_b, x2
        );

        let (lamports_out_b, x2_after_sell) = curve.sell_exact_input(x2, tokens_out_b).unwrap();
        x2 = x2_after_sell;
        println!(
            "(B) Sold {} tokens, got back {} base_tokens, new x={}",
            tokens_out_b, lamports_out_b, x2
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
    ///
    /// For y(x) = A - (K / (C + x)), the local slope dX/dY is:
    ///     dX/dY = (C + x)^2 / K
    /// Here, we pass `x` in from the test context.
    fn approximate_price(curve: &SmoothBondingCurve, x: u64) -> f64 {
        let denom = (curve.c_bonding_scale_factor as f64) + (x as f64);
        let k = curve.k_virtual_pool_offset as f64;
        (denom * denom) / k
    }

    #[test]
    fn test_buy_until_70k_liquidity() {
        let curve = default_curve();
        let target_liquidity_usd = 70_000.0;
        let sol_price_usd = 250.0;
        let target_sol_in_pool = target_liquidity_usd / sol_price_usd;

        // Start with an empty pool
        let mut x: u64 = 0;

        let base_in_per_step: u64 = LAMPORTS_PER_SOL; // 1 SOL per iteration
        let max_iterations: u16 = 1000;
        let mut iteration = 0;

        // Keep buying until x (in SOL) ~ target_sol_in_pool
        while (x as f64) / (LAMPORTS_PER_SOL as f64) < target_sol_in_pool {
            iteration += 1;
            if iteration > max_iterations {
                panic!("Exceeded max iterations; something might be off.");
            }

            let (minted, new_x) = curve.buy_exact_input(x, base_in_per_step).unwrap();
            x = new_x;

            let total_pool_sol = (x as f64) / (LAMPORTS_PER_SOL as f64);
            let new_price = approximate_price(&curve, x);

            println!(
                "Iteration {}: +1 SOL => minted {} tokens, approx price={:.2e}, total SOL={:.4}",
                iteration, minted, new_price, total_pool_sol
            );
        }

        let final_sol = (x as f64) / (LAMPORTS_PER_SOL as f64);
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
