use super::traits::BondingCurveTrait;

/// A smooth bonding curve tracking the total base asset (e.g., SOL, XBT) deposited.
///
/// Formula: y(x) = A - K / (C + x)
/// - A = 1,073,000,191
/// - K = 32,190,005,730
/// - C = 30
/// - x = total base asset contributed so far
#[derive(Debug, Clone)]
pub struct SmoothBondingCurve {
    pub a: f64, // Asymptotic total token supply (A)
    pub k: f64, // Controls how quickly we approach A (K)
    pub c: f64, // Virtual pool – offset to avoid division by zero (C)
    pub x: f64, // Total base asset deposited so far
}

impl SmoothBondingCurve {
    /// Returns the *marginal* price at the current state `x`.
    /// Mathematically: price = dX/dY = (C + x)^2 / K.
    pub fn _price(&self) -> f64 {
        let denom = self.c + self.x;
        (denom * denom) / self.k
    }

    /// Calculates the cumulative minted token supply at `x`.
    fn y_of_x(&self, x_val: f64) -> f64 {
        self.a - self.k / (self.c + x_val)
    }
}

impl BondingCurveTrait for SmoothBondingCurve {
    fn buy_exact_input(&mut self, base_in: f64) -> f64 {
        let old_y = self.y_of_x(self.x);
        let new_y = self.y_of_x(self.x + base_in);
        let minted = new_y - old_y;
        self.x += base_in;
        minted
    }

    fn buy_exact_output(&mut self, amount_out: f64) -> f64 {
        let old_y = self.y_of_x(self.x);
        let target_y = old_y + amount_out;
        let x_prime = (self.k / (self.a - target_y)) - self.c;
        let cost = x_prime - self.x;
        self.x = x_prime;
        cost
    }

    fn sell_exact_input(&mut self, tokens_in: f64) -> f64 {
        let old_y = self.y_of_x(self.x);
        let new_y = old_y - tokens_in;
        let x_prime = (self.k / (self.a - new_y)) - self.c;
        let base_out = self.x - x_prime;
        self.x = x_prime;
        base_out
    }

    fn sell_exact_output(&mut self, amount_out: f64) -> f64 {
        let old_y = self.y_of_x(self.x);
        let new_x = self.x - amount_out;
        let new_y = self.y_of_x(new_x);
        let tokens_in = old_y - new_y;
        self.x = new_x;
        tokens_in
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_exact_input() {
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };
        let amount_in = 1.0;
        let minted = curve.buy_exact_input(amount_in);
        assert!(
            minted > 34_600_000.0 && minted < 34_700_000.0,
            "Minted out of range: {}",
            minted
        );
    }

    #[test]
    fn test_buy_exact_output() {
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };
        let desired_out = 50_000_000.0;
        let cost = curve.buy_exact_output(desired_out);
        let mut fresh_curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };
        let minted_check = fresh_curve.buy_exact_input(cost);
        assert!(
            (minted_check - desired_out).abs() < 1.0,
            "Difference too large: minted_check={}, expected={}",
            minted_check,
            desired_out
        );
    }

    #[test]
    fn test_sell_exact_input() {
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };
        let minted = curve.buy_exact_input(10.0);
        let tokens_in = minted / 2.0;
        let base_out = curve.sell_exact_input(tokens_in);
        assert!(
            base_out > 0.0,
            "Should receive a positive amount of base asset"
        );
    }

    #[test]
    fn test_sell_exact_output() {
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };
        curve.buy_exact_input(100.0);
        let desired_out = 10.0;
        let tokens_in = curve.sell_exact_output(desired_out);
        let mut fresh_curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };
        fresh_curve.buy_exact_input(100.0);
        let base_got = fresh_curve.sell_exact_input(tokens_in);
        assert!(
            (base_got - desired_out).abs() < 1e-6,
            "Difference too large: got={}, expected={}",
            base_got,
            desired_out
        );
    }

    /// This test demonstrates that the marginal price increases
    /// after a user buys some tokens.
    #[test]
    fn test_price_increases_after_buy() {
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };

        let initial_price = curve._price();
        curve.buy_exact_input(10.0);
        let new_price = curve._price();

        // We expect the new price to be higher, because x is larger.
        assert!(
            new_price > initial_price,
            "Price should increase after buying. initial={}, new={}",
            initial_price,
            new_price
        );
    }

    #[test]
    fn test_buy_until_70k_liquidity() {
        let mut curve = SmoothBondingCurve {
            a: 1_073_000_191.0,
            k: 32_190_005_730.0,
            c: 30.0,
            x: 0.0,
        };

        let target_liquidity_usd = 70_000.0;
        let sol_price_usd = 250.0;
        let base_in: f64 = 1.0;
        let iters: u16 = 1000;
        // If 1 SOL costs $250, then for $70,000 we need 70,000/250 = 280 SOL in total.
        let target_sol_in_pool = target_liquidity_usd / sol_price_usd;

        let mut iteration = 0;
        while curve.x < target_sol_in_pool {
            iteration += 1;
            // Buy exactly 1 SOL worth of tokens each step
            let minted = curve.buy_exact_input(base_in);
            let new_price = curve._price();
            let total_pool_sol = curve.x;

            println!(
                "Iteration {iteration}: bought 1 SOL => minted {minted:.2} tokens, \
                 new marginal price = {:.2e}, total SOL in pool = {:.2}",
                new_price, total_pool_sol
            );

            if iteration > iters {
                panic!("Too many iterations, something might be wrong.");
            }
        }

        let final_liquidity_sol = curve.x;
        let final_liquidity_usd = final_liquidity_sol * sol_price_usd;
        println!(
            "Reached ~70k USD liquidity:\n  - Final SOL in pool: {:.2}\n  - Final USD value: ${:.2}\n",
            final_liquidity_sol, final_liquidity_usd
        );

        assert!(
            final_liquidity_usd >= 70_000.0,
            "Expected at least $70k in the pool, but got {}",
            final_liquidity_usd
        );
    }
}
