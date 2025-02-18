use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};

use crate::curves::SmoothBondingCurve;

pub static DECIMALS: u8 = 9;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitCoreParams {
    pub admin: Pubkey,
    pub grad_threshold: u16,
    pub bonding_curve: SmoothBondingCurve,
    pub accepted_base_mint: Pubkey,
    pub graduate_dollars_amount: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}
