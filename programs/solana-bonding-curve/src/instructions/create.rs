use anchor_lang::prelude::*;
use anchor_lang::system_program;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::curves::SmoothBondingCurve;
use crate::{omni_params, OwnedToken};

#[derive(Accounts)]
pub struct CreateToken<'info> {
    /// CHECK: arbitrary seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"omni_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        space = OwnedToken::LEN
    )]
    pub omni_token: Account<'info, OwnedToken>,

    #[account(
        init,
        payer = creator,
        mint::decimals = omni_params::DECIMALS,
        mint::authority = omni_token
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

    // Not used in create_token, but typically
    // required by the same client code flow
    /// CHECK: Escrow account
    pub escrow_pda: UncheckedAccount<'info>,

    // System
    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

pub fn create_token_instruction(ctx: Context<CreateToken>) -> Result<()> {
    let omni_token = &mut ctx.accounts.omni_token;
    omni_token.supply = omni_params::TOTAL_TOKENS;

    omni_token.bonding_curve = SmoothBondingCurve {
        a: omni_params::TOTAL_TOKENS,
        k: omni_params::BONDING_SCALE_FACTOR,
        c: omni_params::VIRTUAL_POOL_OFFSET,
        x: 0,
    };

    Ok(())
}
