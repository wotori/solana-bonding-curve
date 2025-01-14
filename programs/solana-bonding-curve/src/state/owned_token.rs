// state/owned_token.rs
use anchor_lang::prelude::*;

use super::{BondingCurveCoefficients, TargetChain};

/// Account to store essential bonding-curve information for a token.
/// (We're NOT storing description, links, etc. here, since Metaplex
/// Metadata can handle that for wallet display.)
#[account]
pub struct OwnedToken {
    /// A short name for the token (for internal use in program)
    pub token_name: String,

    /// A short ticker symbol (for internal use in program)
    pub ticker: String,

    /// Total supply (planned or maximum) for the token
    pub supply: u64,

    /// How many tokens to mint initially for the owner
    pub initial_buy_amount: u64,

    /// The initial buy price used in bonding curve logic
    pub initial_buy_price: f64,

    /// Possible target chains for bridging or cross-chain usage
    pub target_chains: Vec<TargetChain>,

    /// Public token (e.g., SOL or another SPL token) used for liquidity pool
    pub public_token: Pubkey,

    /// Bonding curve coefficients for price calculations
    pub bonding_curve_coefficients: BondingCurveCoefficients,

    /// URI
    pub metadata_uri: String,
}

impl OwnedToken {
    pub const LEN: usize = 8          // discriminator
        + 32                          // token_name (assume 32 chars max)
        + 10                          // ticker (assume 10 chars max)
        + 8                           // supply
        + 8                           // initial_buy_amount
        + 8                           // initial_buy_price
        + 4                           // length of target_chains vec
        + (4 + 32) * 2                // public_token + bonding_curve_coefficients (simplified)
        + 24; // buffer for future expansions
}
