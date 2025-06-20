use anchor_lang::prelude::*;

mod curves;
mod errors;
mod events;
mod utils;
mod xyber_params;

mod instructions;

use crate::xyber_params::{InitCoreParams, TokenParams};
use curves::SmoothBondingCurve;
use instructions::*;

declare_id!("8FydojysL5DJ8M3s15JLFEbsKzyQ1BcFgSMVDvJetEEq");

/// The sixbte, global state for all tokens.
#[account]
pub struct XyberCore {
    pub admin: Pubkey,
    pub grad_threshold: u64,
    pub total_supply: u64,
    // The bonding curve shared by all tokens
    pub bonding_curve: SmoothBondingCurve,
    pub accepted_base_mint: Pubkey,

    /// Treasury account for protocol fees
    pub treasury_wallet: Pubkey,

    /// Commission rate (in basis points)
    pub commission_rate: u64,
}

impl XyberCore {
    pub const LEN: usize = 8  // Anchor discriminator
        + (1 + 32) // admin
        + (1 + 8)  // grad_threshold
        + (1 + 8)  // total_supply
        + (1 + 40) // bonding_curve
        + (1 + 32) // accepted_base_mint
        + (1 + 32) // treasury
        + (1 + 8); // commission_rate
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

    /// Agent's commission account (mint Pubkey)
    pub agent_wallet_pubkey: Pubkey,

    // used for managing grad_threshold from XyberCore
    pub total_chains: u8,
}

impl XyberToken {
    pub const LEN: usize = 8  // Discriminator
        + 1   // is_graduated
        + 32  // mint
        + 32  // vault
        + 32  // creator
        + 1   // total_chains
        + 32  // agent_wallet_pubkey
        ;
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
