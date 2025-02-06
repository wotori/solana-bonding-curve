use crate::curves::SmoothBondingCurve;
use crate::xyber_params::CreateTokenParams; // Your parameters struct
use crate::XyberToken;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(params: CreateTokenParams)]
pub struct InitTokenCore<'info> {
    /// CHECK: This account is used solely for PDA derivation.
    pub token_seed: AccountInfo<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        space = XyberToken::LEN
    )]
    pub xyber_token: Account<'info, XyberToken>,

    pub system_program: Program<'info, System>,
}

pub fn init_token_core_instruction(
    ctx: Context<InitTokenCore>,
    params: CreateTokenParams,
) -> Result<()> {
    let token = &mut ctx.accounts.xyber_token;
    token.accepted_base_mint = params.accepted_base_mint;
    token.bonding_curve = SmoothBondingCurve {
        a_total_tokens: params.bonding_curve.a_total_tokens,
        k_virtual_pool_offset: params.bonding_curve.k_virtual_pool_offset,
        c_bonding_scale_factor: params.bonding_curve.c_bonding_scale_factor,
        x_total_base_deposit: 0,
    };
    token.admin = params.admin;
    token.graduate_dollars_amount = params.graduate_dollars_amount;
    token.is_graduated = false;
    token.mint = Pubkey::default();
    token.vault = Pubkey::default();
    Ok(())
}
