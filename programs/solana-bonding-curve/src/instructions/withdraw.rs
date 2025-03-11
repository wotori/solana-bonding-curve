use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// use crate::errors::CustomError;
use crate::{XyberCore, XyberToken};

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    #[account(
        mut,
        seeds = [b"xyber_core"],
        bump
    )]
    pub xyber_core: Account<'info, XyberCore>,

    /// CHECK: admin from xyber_core
    #[account(
        address = xyber_core.admin,
        mut,
        signer
    )]
    pub admin: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"xyber_token", token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Account<'info, XyberToken>,

    /// CHECK: Used only for PDA seed derivation
    pub token_seed: UncheckedAccount<'info>,

    /// CHECK: Used only for PDA seed derivation
    pub creator: UncheckedAccount<'info>,

    /// Escrow token account holding the payment tokens (e.g. USDC)
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        address = xyber_core.accepted_base_mint
    )]
    pub base_token_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = base_token_mint,
        associated_token::authority = admin
    )]
    pub admin_token_account: Account<'info, TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
    // TODO: restore this in production or keep?
    // require!(
    //     ctx.accounts.xyber_token.is_graduated,
    //     CustomError::LiquidityNotGraduated
    // );

    let bump = ctx.bumps.xyber_token;
    let seeds = &[
        b"xyber_token".as_ref(),
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
