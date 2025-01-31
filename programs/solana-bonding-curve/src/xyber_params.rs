use anchor_lang::{
    prelude::Pubkey, solana_program::native_token::LAMPORTS_PER_SOL, AnchorDeserialize,
    AnchorSerialize,
};

use crate::curves::SmoothBondingCurve;

pub static DECIMALS: u8 = 9;

pub static _TOTAL_TOKENS: u64 = 1_073_000_191;
pub static _VIRTUAL_POOL_OFFSET: u64 = 30 * LAMPORTS_PER_SOL;
pub static _BONDING_SCALE_FACTOR: u128 = 32_190_005_730 * (LAMPORTS_PER_SOL as u128);

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateTokenParams {
    pub token_supply: u64,
    // pub token_decimals: u8,
    pub token_grad_thr_usd: u16,

    pub bonding_curve: SmoothBondingCurve,
    pub accepted_base_mint: Pubkey,
}

/*  TODO: Theoretically, token_supply parameter is present in the bonding curve as a_total_tokens, but for reliability,
it’s better to calculate it here as well. It might be worth considering removing it.*/
