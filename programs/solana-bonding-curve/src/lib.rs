use anchor_lang::prelude::*;

mod curves;
mod errors;
mod events;
mod xyber_params;

mod instructions;

use crate::xyber_params::{InitCoreParams, TokenParams};
use curves::SmoothBondingCurve;
use instructions::*;

// current development id is BdHFqKoxuP3nFChJU7uLx39CJMF88SxH5ZkX4oZ5YqcD
// new EHvS7Ts5k8Dvd8kXAyYYq2CmFVrLFyesTzuRC6D5KcTg
declare_id!("EHvS7Ts5k8Dvd8kXAyYYq2CmFVrLFyesTzuRC6D5KcTg");

/// The sixbte, global state for all tokens.
#[account]
pub struct XyberCore {
    // Who is allowed to update contract parameters
    pub admin: Pubkey,

    // Example: A sixbte global threshold
    pub grad_threshold: u64,

    // The bonding curve shared by all tokens
    pub bonding_curve: SmoothBondingCurve,

    // Base mint accepted as payment (e.g. XBT SPL Token)
    pub accepted_base_mint: Pubkey,
}

impl XyberCore {
    pub const LEN: usize = 8  // Anchor discriminator (1 + X -> 1 stand for optional fields)
        + (1 + 32) // admin (Pubkey)
        + (1 + 8)  // grad_threshold (u16)
        // SmoothBondingCurve has 4 fields:
        // a_total_tokens: u64 -> 8 bytes
        // k_virtual_pool_offset: u128 -> 16 bytes
        // c_bonding_scale_factor: u64 -> 8 bytes
        // In total: 8 + 16 + 8 + 8 = 32
        + (1 + 40)  // bonding_curve
        + (1 + 32); // accepted_base_mint (Pubkey)
}

/// One account per unique token. It holds only “token-specific” info.
#[account]
pub struct XyberToken {
    // Per-token graduation boolean
    pub is_graduated: bool,

    // The mint for this token
    pub mint: Pubkey,

    // The vault that holds the minted tokens
    pub vault: Pubkey,

    // The creator's Pubkey
    pub creator: Pubkey,

    // used for managing grad_threshold from XyberCore
    pub total_chains: u8,
}

impl XyberToken {
    pub const LEN: usize = 8  // Discriminator
        + 1  // is_graduated
        + 32  // mint
        + 32  // vault
        + 32 // creator
        + 1; // total_chains
}

#[program]
pub mod bonding_curve {
    use super::*;

    // SETUP OR UPDATE XYBER CORE PARAMETERS
    pub fn update_xyber_core_instruction(
        ctx: Context<UpdateXyberCore>,
        params: InitCoreParams,
    ) -> Result<()> {
        instructions::update_xyber_core_instruction(ctx, params)
    }

    // 1.1 CREATE TOKEN
    pub fn mint_full_supply_instruction(
        ctx: Context<InitAndMint>,
        params: TokenParams,
    ) -> Result<()> {
        instructions::mint_full_supply_instruction(ctx, params)
    }

    pub fn buy_exact_input_instruction(
        ctx: Context<BuyToken>,
        base_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::buy_exact_input_instruction(ctx, base_in, min_amount_out)
    }

    pub fn buy_exact_output_instruction(
        ctx: Context<BuyToken>,
        tokens_out: u64,
        max_payment_amount: u64,
    ) -> Result<()> {
        instructions::buy_exact_output_instruction(ctx, tokens_out, max_payment_amount)
    }

    pub fn sell_exact_input_instruction(
        ctx: Context<SellToken>,
        normalized_token_amount: u64,
        min_base_amount_out: u64,
    ) -> Result<()> {
        instructions::sell_exact_input_instruction(
            ctx,
            normalized_token_amount,
            min_base_amount_out,
        )
    }

    pub fn sell_exact_output_instruction(
        ctx: Context<SellToken>,
        lamports: u64,
        max_tokens_in: u64,
    ) -> Result<()> {
        instructions::sell_exact_output_instruction(ctx, lamports, max_tokens_in)
    }

    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>) -> Result<()> {
        instructions::withdraw_liquidity(ctx)
    }

    pub fn close_xyber_core_instruction(_ctx: Context<CloseXyberCore>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CloseXyberCore<'info> {
    #[account(mut, has_one = admin, close = admin)]
    pub xyber_core: Account<'info, XyberCore>,
    #[account(mut)]
    pub admin: Signer<'info>,
}
