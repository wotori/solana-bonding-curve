use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};

use crate::curves::SmoothBondingCurve;

pub static DECIMALS: u8 = 9;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitCoreParams {
    pub admin: Option<Pubkey>,
    pub grad_threshold: Option<u64>,
    pub bonding_curve: Option<SmoothBondingCurve>,
    pub accepted_base_mint: Option<Pubkey>,
    pub total_supply: Option<u64>
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,

    pub total_chains: u8,
}
