use anchor_lang::prelude::*;
use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};

use super::{BondingCurveCoefficients, TargetChain};

/// Structure for token launch parameters
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TokenLaunchParams {
    pub token_name: String,
    pub ticker: String,
    pub media_url: String,
    pub description: String,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub supply: u64,
    pub initial_buy_amount: u64,
    pub initial_buy_price: f64,
    pub target_chains: Vec<TargetChain>,
    pub public_token: Pubkey,
    pub bonding_curve_coefficients: BondingCurveCoefficients,
}
