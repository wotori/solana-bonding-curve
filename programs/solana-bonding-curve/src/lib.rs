use anchor_lang::prelude::*;

mod curves;
mod errors;
mod events;
mod xyber_params;

mod instructions;

use crate::xyber_params::CreateTokenParams;
use curves::SmoothBondingCurve;
use instructions::*;

declare_id!("GH5c6RihMrseCqDGToAmEEAHGkdSSU9MSdrsaghnErCV");

#[account]
pub struct XyberToken {
    pub grad_threshold: u16,
    pub bonding_curve: SmoothBondingCurve,
    pub accepted_base_mint: Pubkey,
    pub admin: Pubkey,

    pub graduate_dollars_amount: u32,
    pub is_graduated: bool,

    pub mint: Pubkey,
    pub vault: Pubkey,
}

impl XyberToken {
    pub const LEN: usize = 8  // Discriminator
        + 2   // grad_threshold (u16)
        + 40  // bonding_curve (struct)
        + 32  // accepted_base_mint (Pubkey)
        + 32  // admin (Pubkey)
        + 4   // graduate_dollars_amount
        + 1   // is_graduated (bool)
        
        + 32  // mint (Pubkey)
        + 32; // vault (Pubkey)
}

#[program]
pub mod bonding_curve {

    use super::*;

    // 1.0 CREATE TOKEN
    pub fn init_core_instruction(
        ctx: Context<InitTokenCore>,
        params: CreateTokenParams,
    ) -> Result<()> {
        instructions::init_token_core_instruction(ctx, params)
    }

    // 1.1 CREATE TOKEN
    pub fn init_and_mint_full_supply_instruction(
        ctx: Context<InitAndMint>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::init_and_mint_full_supply_instruction(ctx, name, symbol, uri)
    }

    // 1.2 MINT INITIAL TOKENS
    pub fn mint_initial_tokens_instruction(
        ctx: Context<MintInitialTokens>,
        deposit_lamports: u64,
    ) -> Result<()> {
        instructions::mint_initial_tokens_instruction(ctx, deposit_lamports)
    }

    pub fn buy_exact_input_instruction(ctx: Context<BuyToken>, base_in: u64,min_amount_out: u64) -> Result<()> {
        instructions::buy_exact_input_instruction(ctx, base_in, min_amount_out)
    }

    pub fn buy_exact_output_instruction(ctx: Context<BuyToken>, tokens_out: u64, max_payment_amount: u64) -> Result<()> {
        instructions::buy_exact_output_instruction(ctx, tokens_out, max_payment_amount)
    }

    pub fn sell_exact_input_instruction(
        ctx: Context<SellToken>,
        normalized_token_amount: u64,
    ) -> Result<()> {
        instructions::sell_exact_input_instruction(ctx, normalized_token_amount)
    }

    pub fn sell_exact_output_instruction(ctx: Context<SellToken>, lamports: u64) -> Result<()> {
        instructions::sell_exact_output_instruction(ctx, lamports)
    }

    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
        instructions::withdraw_liquidity(ctx, amount)
    }
}
