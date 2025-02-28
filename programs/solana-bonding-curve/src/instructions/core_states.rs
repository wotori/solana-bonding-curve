use crate::curves::SmoothBondingCurve;
use crate::xyber_params::InitCoreParams;
use crate::XyberCore;
use anchor_lang::prelude::*;

fn fill_core_fields(core: &mut XyberCore, params: &InitCoreParams) {
    core.admin = params.admin;
    core.grad_threshold = params.grad_threshold;
    core.accepted_base_mint = params.accepted_base_mint;
    core.bonding_curve = SmoothBondingCurve {
        a_total_tokens: params.bonding_curve.a_total_tokens,
        k_virtual_pool_offset: params.bonding_curve.k_virtual_pool_offset,
        c_bonding_scale_factor: params.bonding_curve.c_bonding_scale_factor,
    };
    core.graduate_dollars_amount = params.graduate_dollars_amount;
}

#[derive(Accounts)]
pub struct InitXyberCore<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [b"xyber_core"],
        bump,
        space = XyberCore::LEN
    )]
    pub xyber_core: Account<'info, XyberCore>,

    pub system_program: Program<'info, System>,
}

pub fn setup_xyber_core_instruction(
    ctx: Context<InitXyberCore>,
    params: InitCoreParams,
) -> Result<()> {
    let core = &mut ctx.accounts.xyber_core;
    fill_core_fields(core, &params);

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateXyberCore<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, has_one = admin)]
    pub xyber_core: Account<'info, XyberCore>,
}

pub fn update_xyber_core_instruction(
    ctx: Context<UpdateXyberCore>,
    params: InitCoreParams,
) -> Result<()> {
    let core = &mut ctx.accounts.xyber_core;
    fill_core_fields(core, &params);

    Ok(())
}
