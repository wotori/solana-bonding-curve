use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
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

    /// CHECK: Admin from xyber_core
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

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = xyber_token
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

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

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = admin
    )]
    pub admin_vault_account: Account<'info, TokenAccount>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>) -> Result<()> {
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

    // 1) Transfer the base tokens from escrow to the admin’s base ATA
    let escrow_balance = ctx.accounts.escrow_token_account.amount;
    let cpi_ctx_escrow = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.admin_token_account.to_account_info(), // Admin’s base token ATA
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer,
    );
    token::transfer(cpi_ctx_escrow, escrow_balance)?;

    // 2) Transfer the project tokens from vault to the admin’s project ATA
    let vault_balance = ctx.accounts.vault_token_account.amount;
    let cpi_ctx_vault = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(), // Vault = project token
            to: ctx.accounts.admin_vault_account.to_account_info(),   // Admin’s project token ATA
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer,
    );
    token::transfer(cpi_ctx_vault, vault_balance)?;

    Ok(())
}
