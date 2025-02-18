use anchor_lang::prelude::*;

mod curves;
mod errors;
mod events;
mod xyber_params;

mod instructions;

use crate::xyber_params::{InitCoreParams, TokenParams};
use curves::SmoothBondingCurve;
use instructions::*;

declare_id!("7TtWm2z8uixrGbxhkT1SYZfWfbiAJEg7zRaozUh46v2C");

/// The sixbte, global state for all tokens.
#[account]
pub struct XyberCore {
    // Who is allowed to update contract parameters
    pub admin: Pubkey,

    // Example: A sixbte global threshold
    pub grad_threshold: u16,

    // The bonding curve shared by all tokens
    pub bonding_curve: SmoothBondingCurve,

    // Base mint accepted as payment (e.g. XBT SPL Token)
    pub accepted_base_mint: Pubkey,

    // The USD threshold at which any token is considered “graduated”
    pub graduate_dollars_amount: u32,
}

impl XyberCore {
    pub const LEN: usize = 8  // Anchor discriminator
        + 32 // admin (Pubkey)
        + 2  // grad_threshold (u16)
        // SmoothBondingCurve has 4 fields:
        // a_total_tokens: u64 -> 8 bytes
        // k_virtual_pool_offset: u128 -> 16 bytes
        // c_bonding_scale_factor: u64 -> 8 bytes
        // x_total_base_deposit: u64 -> 8 bytes
        // In total: 8 + 16 + 8 + 8 = 40
        + 40  // bonding_curve
        + 32  // accepted_base_mint (Pubkey)
        + 4; // graduate_dollars_amount (u32)
}

/// One account per unique token. It holds only “token-specific” info.
#[account]
pub struct XyberToken {
    // Which XyberCore does this token reference?
    pub xyber_core: Pubkey,

    // Per-token graduation boolean
    pub is_graduated: bool,

    // The mint for this token
    pub mint: Pubkey,

    // The vault that holds the minted tokens
    pub vault: Pubkey,
}

impl XyberToken {
    pub const LEN: usize = 8  // Discriminator
        + 32  // xyber_core
        + 1  // is_graduated
        + 32  // mint
        + 32; // vault
}
#[program]
pub mod bonding_curve {
    use super::*;

    // SETUP XYBER CORE PARAMETERS
    pub fn setup_xyber_core_instruction(
        ctx: Context<InitXyberCore>,
        params: InitCoreParams,
    ) -> Result<()> {
        instructions::setup_xyber_core_instruction(ctx, params)
    }

    // UPDATE XYBER CORE PARAMETERS
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

    // 1.2 MINT INITIAL TOKENS
    pub fn initial_buy_tokens_instruction(
        ctx: Context<MintInitialTokens>,
        deposit_lamports: u64,
    ) -> Result<()> {
        instructions::initial_buy_tokens_instruction(ctx, deposit_lamports)
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
