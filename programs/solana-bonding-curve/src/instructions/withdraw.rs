use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::CustomError;
use crate::{XyberCore, XyberToken};

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    /// Admin allowed to withdraw liquidity
    #[account(mut)]
    /// CHECK: admin account
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin,
        seeds = [b"xyber_core"],
        bump
    )]
    pub xyber_core: Account<'info, XyberCore>,

    #[account(
        mut,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Account<'info, XyberToken>,

    /// CHECK: Used only for PDA seed derivation
    pub token_seed: UncheckedAccount<'info>,

    /// CHECK: Used only for PDA seed derivation
    pub creator: UncheckedAccount<'info>,

    /// Escrow token account holding the payment tokens (e.g. XBT)
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// Admin's token account to receive the withdrawn tokens
    #[account(mut)]
    pub admin_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
    require!(
        ctx.accounts.xyber_token.is_graduated,
        CustomError::LiquidityNotGraduated
    );

    // Derive PDA signer seeds; PDA was created with seeds:
    // [b"xyber_token", creator.key, token_seed.key, bump]
    let bump = ctx.bumps.xyber_token;
    let seeds = &[
        b"xyber_token".as_ref(),
        ctx.accounts.creator.key.as_ref(),
        ctx.accounts.token_seed.key.as_ref(),
        &[bump],
    ];
    let signer = &[&seeds[..]];

    // Transfer tokens from escrow token account to admin's token account
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.admin_token_account.to_account_info(),
            // The authority is the PDA (xyber_token), which signs using the provided seeds
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer,
    );
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}
