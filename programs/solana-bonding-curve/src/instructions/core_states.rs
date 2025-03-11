use crate::{xyber_params::InitCoreParams, XyberCore};
use anchor_lang::prelude::*;

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

    pub system_program: Program<'info, System>,
}

pub fn update_xyber_core_instruction(
    ctx: Context<UpdateXyberCore>,
    params: InitCoreParams,
) -> Result<()> {
    fill_core_fields(&mut ctx.accounts.xyber_core, &params);
    Ok(())
}
