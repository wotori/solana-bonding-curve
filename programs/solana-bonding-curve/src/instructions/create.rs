use anchor_lang::prelude::*;
use anchor_lang::{solana_program::native_token::LAMPORTS_PER_SOL, system_program};

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
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        space = OwnedToken::LEN
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(
        init,
        payer = creator,
        mint::decimals = omni_params::DECIMALS,
        mint::authority = owned_token
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
    let owned_token = &mut ctx.accounts.owned_token;
    owned_token.supply = 1_073_000_191;

    owned_token.bonding_curve = SmoothBondingCurve {
        a: 1_073_000_191,
        k: 32_190_005_730 * LAMPORTS_PER_SOL as u128,
        c: 30 * LAMPORTS_PER_SOL,
        x: 0,
    };

    Ok(())
}
