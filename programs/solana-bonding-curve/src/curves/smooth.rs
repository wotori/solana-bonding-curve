use super::traits::BondingCurveTrait;
use anchor_lang::prelude::*;

/// A smooth bonding curve tracking the total base asset (e.g., SOL, XBT) deposited.
///
/// Formula: y(x) = A - K / (C + x)
/// - `A` = asymptotic max token supply (in integer "token units")
/// - `K` = dimension is (token * lamport), controlling how quickly we approach A
/// - `C` = virtual pool offset (in lamports), e.g. 30 SOL -> 30_000_000_000 lamports
/// - `x` = total base asset deposited so far (in lamports)
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SmoothBondingCurve {
    pub a: u64,  // Asymptotic total token supply (in "raw" tokens)
    pub k: u128, // Controls how quickly we approach A (token * lamport)
    pub c: u64,  // Virtual pool offset (in lamports)
    pub x: u64,  // Total base asset deposited (in lamports)
}

impl SmoothBondingCurve {
    /// Calculates the total minted tokens at `x_val` lamports in the pool:
    /// y(x) = A - (K / (C + x))   (all integer math)
    fn y_of_x(&self, x_val: u64) -> u128 {
        // c + x (in lamports), promoted to 128
        let denom = (self.c as u128).saturating_add(x_val as u128);
        // k / denom => yields "tokens" in integer
        let k_over_denom = self.k.saturating_div(denom);
        // A - k_over_denom => also "tokens"
        // a is u64 (fits in 128), k_over_denom is u128
        let a_minus = (self.a as u128).saturating_sub(k_over_denom);
        a_minus
    }
}

impl BondingCurveTrait for SmoothBondingCurve {
    /// pass exact lamports, return exact tokens minted
    fn buy_exact_input(&mut self, base_in: u64) -> u128 {
        let old_y = self.y_of_x(self.x);
        let new_y = self.y_of_x(self.x.saturating_add(base_in));
        let minted = new_y.saturating_sub(old_y);
        self.x = self.x.saturating_add(base_in);
        minted
    }

    /// pass exact tokens in, return exact lamports out
    fn sell_exact_input(&mut self, tokens_in: u128) -> u64 {
        let old_y = self.y_of_x(self.x);
        // new_y = old_y - tokens_in
        let new_y = old_y
            .checked_sub(tokens_in)
            .expect("Cannot sell more tokens than owned by curve state");

        // Solve for x' in: new_y = A - (K / (C + x')) => (K / (C + x')) = A - new_y
        let a_minus_ny = (self.a as u128).saturating_sub(new_y);
        let big_val = self.k.saturating_div(a_minus_ny); // = C + x'
        if big_val < (self.c as u128) {
            panic!("Curve underflow: c + x' is negative");
        }

        let x_prime = big_val.saturating_sub(self.c as u128);
        let base_out = (self.x as u128).saturating_sub(x_prime);
        self.x = x_prime as u64;
        base_out as u64
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

    #[test]
    fn test_buy_exact_input() {
        // Initialize the bonding curve with example parameters
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191,
            k: 32_190_005_730_000_000_000,
            c: 30_000_000_000,
            x: 0,
        };

        // Buy tokens with 1 SOL (in lamports)
        let base_in = 1 * LAMPORTS_PER_SOL;
        let minted = curve.buy_exact_input(base_in);

        // Ensure the minted token amount is within the expected range
        assert!(
            (34_600_000..34_700_000).contains(&(minted as u64)),
            "Minted tokens out of expected range: {}",
            minted
        );
    }

    #[test]
    fn test_sell_exact_input() {
        // Initialize the bonding curve with example parameters
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191,
            k: 32_190_005_730_000_000_000,
            c: 30_000_000_000,
            x: 0,
        };

        // First, buy tokens with 10 SOL
        let sol_in = 10 * LAMPORTS_PER_SOL;
        let minted_tokens = curve.buy_exact_input(sol_in);

        // Then, sell half of the tokens
        let tokens_to_sell = minted_tokens / 2;
        let lamports_out = curve.sell_exact_input(tokens_to_sell);

        // Ensure that selling tokens results in receiving some lamports
        assert!(
            lamports_out > 0,
            "Should receive some lamports when selling tokens"
        );

        // Note: Selling should return less than 5 SOL due to the increased price
        // after the initial purchase. A more precise test can be added if needed.
    }

    /// A local function to estimate the marginal price (dX/dY) in float,
    /// used for logging and tracking price growth.
    fn approximate_price(curve: &SmoothBondingCurve) -> f64 {
        let denom = (curve.c as f64) + (curve.x as f64);
        let k = curve.k as f64;
        // price = (C + x)^2 / K
        (denom * denom) / k
    }

    /// This test simulates continuous SOL purchases until the total pool liquidity reaches $70k.
    #[test]
    fn test_buy_until_70k_liquidity() {
        // Initialize the bonding curve with example parameters
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191,
            k: 32_190_005_730 * LAMPORTS_PER_SOL as u128,
            c: 30 * LAMPORTS_PER_SOL as u64,
            x: 0,
        };

        let target_liquidity_usd = 70_000.0;
        let sol_price_usd = 250.0;

        // Calculate the target amount of SOL in the pool (280 SOL for $70k liquidity)
        let target_sol_in_pool = target_liquidity_usd / sol_price_usd;

        // Each step buys 1 SOL
        let base_in_per_step: u64 = LAMPORTS_PER_SOL / 1; // 1 SOL

        // Limit the number of iterations
        let max_iterations: u16 = 1000;
        let mut iteration = 0;

        // Continue buying until the total SOL in the pool reaches the target
        while (curve.x as f64) / (LAMPORTS_PER_SOL as f64) < target_sol_in_pool {
            iteration += 1;

            // Buy 1 SOL
            let minted = curve.buy_exact_input(base_in_per_step);
            let new_price = approximate_price(&curve);
            let total_pool_sol = (curve.x as f64) / (LAMPORTS_PER_SOL as f64);

            println!(
                "Iteration {iteration}: bought 1 SOL => minted {minted} tokens, \
             approx price = {:.2e}, total SOL in pool = {:.4}",
                new_price, total_pool_sol
            );

            if iteration > max_iterations {
                panic!("Too many iterations, something might be wrong.");
            }
        }

        let final_liquidity_sol = (curve.x as f64) / (LAMPORTS_PER_SOL as f64);
        let final_liquidity_usd = final_liquidity_sol * sol_price_usd;

        println!(
        "Reached ~70k USD liquidity:\n  - Final SOL in pool: {:.2}\n  - Final USD value: ${:.2}\n",
        final_liquidity_sol, final_liquidity_usd
    );

        // Ensure the final USD liquidity is at least $70k
        assert!(
            final_liquidity_usd >= 70_000.0,
            "Expected at least $70k in the pool, but got ${:.2}",
            final_liquidity_usd
        );
    }
}
