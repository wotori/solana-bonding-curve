pub trait BondingCurveTrait {
    fn buy_exact_input(&mut self, base_in: u64) -> u128;
    fn buy_exact_output(&mut self, amount_in: u128) -> u64;
    fn sell_exact_input(&mut self, amount_in: u128) -> u64;
    fn sell_exact_output(&mut self, amount_in: u128) -> u64;
}
