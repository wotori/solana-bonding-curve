pub trait BondingCurveTrait {
    fn buy_exact_input(&mut self, base_in: u64) -> u128;
    fn sell_exact_input(&mut self, amount_in: u128) -> u64;
}
