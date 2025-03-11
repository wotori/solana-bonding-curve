use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{xyber_params::InitCoreParams, XyberCore};

pub fn fill_core_fields(core: &mut XyberCore, params: &InitCoreParams) {
    if let Some(admin) = params.admin {
        core.admin = admin;
    }
    if let Some(grad_threshold) = params.grad_threshold {
        core.grad_threshold = grad_threshold;
    }
    if let Some(bonding_curve) = &params.bonding_curve {
        core.bonding_curve = bonding_curve.clone();
    }
    if let Some(accepted_base_mint) = params.accepted_base_mint {
        core.accepted_base_mint = accepted_base_mint;
    }
}

#[derive(Accounts)]
pub struct UpdateXyberCore<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"xyber_core"],
        bump,
        space = XyberCore::LEN
    )]
    pub xyber_core: Account<'info, XyberCore>,

    #[account(mut)]
    pub new_accepted_base_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = new_accepted_base_mint,
        associated_token::authority = xyber_core
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn update_xyber_core_instruction(
    ctx: Context<UpdateXyberCore>,
    params: InitCoreParams,
) -> Result<()> {
    fill_core_fields(&mut ctx.accounts.xyber_core, &params);
    Ok(())
}
