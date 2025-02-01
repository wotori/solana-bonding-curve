use anchor_lang::prelude::*;
use anchor_lang::system_program;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::curves::SmoothBondingCurve;
use crate::xyber_params::CreateTokenParams;
use crate::{xyber_params, XyberToken};

#[derive(Accounts)]
#[instruction(params: CreateTokenParams)]
pub struct CreateToken<'info> {
    /// CHECK: seed account
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        space = XyberToken::LEN
    )]
    pub xyber_token: Box<Account<'info, XyberToken>>,

    #[account(
        init,
        payer = creator,
        mint::decimals = xyber_params::DECIMALS,
        mint::authority = xyber_token
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    // Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

/// Creates a new token
pub fn init_token_instruction(
    ctx: Box<Context<CreateToken>>,
    params: CreateTokenParams,
) -> Result<()> {
    let xyber_token = &mut ctx.accounts.xyber_token;

    xyber_token.supply = params.bonding_curve.a_total_tokens;
    xyber_token.grad_threshold = params.token_grad_thr_usd;
    xyber_token.accepted_base_mint = params.accepted_base_mint;

    xyber_token.bonding_curve = SmoothBondingCurve {
        a_total_tokens: params.bonding_curve.a_total_tokens,
        k_virtual_pool_offset: params.bonding_curve.k_virtual_pool_offset,
        c_bonding_scale_factor: params.bonding_curve.c_bonding_scale_factor,
        x_total_base_deposit: 0,
    };

    Ok(())
}
