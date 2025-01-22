pub trait BondingCurveTrait {
    fn buy_exact_input(&mut self, base_in: f64) -> f64;
    fn buy_exact_output(&mut self, amount_out: f64) -> f64;
    fn sell_exact_input(&mut self, amount_in: f64) -> f64;
    fn sell_exact_output(&mut self, amount_out: f64) -> f64;
}
