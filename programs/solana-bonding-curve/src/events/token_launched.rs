use anchor_lang::prelude::*;

use crate::state::{BondingCurveCoefficients, TargetChain};

/// https://book.anchor-lang.com/anchor_in_depth/events.html
#[event]
pub struct TokenLaunched {
    pub token_name: String,
    pub ticker: String,
    pub media_url: String,
    pub description: String,
    pub supply: u64,
    pub initial_buy_amount: u64,
    pub initial_buy_price: f64,
    pub target_chains: Vec<TargetChain>,
    pub public_token: Pubkey,
    pub bonding_curve_coefficients: BondingCurveCoefficients,
}
