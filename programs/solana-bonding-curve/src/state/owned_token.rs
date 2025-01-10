use anchor_lang::prelude::*;

use super::{BondingCurveCoefficients, TargetChain};

/// Account to store token information
#[account]
pub struct OwnedToken {
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

impl OwnedToken {
    // Calculate the required space for the account
    pub const LEN: usize = 8
        + 32 + // token_name (assume a maximum of 32 characters)
        10 + // ticker (assume a maximum of 10 characters)
        200 + // media_url (assume a maximum of 200 characters)
        500 + // description (assume a maximum of 500 characters)
        200 + // website (assume a maximum of 200 characters)
        200 + // twitter (assume a maximum of 200 characters)
        200 + // telegram (assume a maximum of 200 characters)
        8 + // supply
        8 + // initial_buy_amount
        8 + // initial_buy_price
        4 + // length of the target_chains array
        (4 + 32) * 2 + // public_token (Pubkey is 32 bytes) and bonding_curve_coefficients
        24; // Additional fields if necessary
}
