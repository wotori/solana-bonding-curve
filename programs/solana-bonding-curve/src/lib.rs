use anchor_lang::prelude::*;

mod curves;
mod errors;
mod xyber_params;

mod instructions;

use crate::xyber_params::CreateTokenParams;
use curves::SmoothBondingCurve;
use instructions::*;

declare_id!("GMjvbDmasN1FyYD6iGfj5u8EETdk9gTQnyoZUQA4PVGT");

#[account]
pub struct XyberToken {
    pub supply: u64,
    pub grad_threshold: u16,
    pub bonding_curve: SmoothBondingCurve,
    pub accepted_base_mint: Pubkey,
    pub admin: Pubkey,
    pub is_graduated: bool,
}

impl XyberToken {
    pub const LEN: usize = 8  // Discriminator
        + 8  // supply (u64)
        + 2  // grad_threshold (u16)
        + 40 // bonding_curve (struct)
        + 32 // escrow_pda (Pubkey)
        + 1  // escrow_bump (u8)
        + 32 // accepted_base_mint (Pubkey)
        + 32 // admin (Pubkey)
        + 1; // is_graduated (bool)
}

#[program]
pub mod bonding_curve {

    use super::*;

    // 1.1 CREATE TOKEN
    pub fn init_token_instruction(
        ctx: Context<CreateToken>,
        params: CreateTokenParams,
    ) -> Result<()> {
        instructions::init_token_instruction(ctx, params)
    }

    // 1.2 MINT INITIAL TOKENS
    pub fn mint_initial_tokens_instruction(
        ctx: Context<MintInitialTokens>,
        deposit_lamports: u64,
    ) -> Result<()> {
        instructions::mint_initial_tokens_instruction(ctx, deposit_lamports)
    }

    // (1.3) SET METADATA
    pub fn set_metadata_instruction(
        ctx: Context<SetMetadata>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::set_metadata_instruction(ctx, name, symbol, uri)
    }

    pub fn buy_exact_input_instruction(ctx: Context<BuyToken>, lamports: u64) -> Result<()> {
        instructions::buy_exact_input_instruction(ctx, lamports)
    }

    pub fn buy_exact_output_instruction(ctx: Context<BuyToken>, tokens_out: u64) -> Result<()> {
        instructions::buy_exact_output_instruction(ctx, tokens_out)
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
