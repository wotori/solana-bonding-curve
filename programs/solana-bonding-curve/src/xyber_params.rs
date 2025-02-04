use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};

use crate::curves::SmoothBondingCurve;

pub static DECIMALS: u8 = 9;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateTokenParams {
    pub token_supply: u64,
    pub token_grad_thr_usd: u16,
    pub bonding_curve: SmoothBondingCurve,
    pub accepted_base_mint: Pubkey,
}
