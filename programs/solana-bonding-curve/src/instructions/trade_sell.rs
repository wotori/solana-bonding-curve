use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::XyberCore;
use crate::XyberToken;

#[derive(Accounts)]
pub struct SellToken<'info> {
    /// CHECK: used as a seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Creator account
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
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

    /// The escrow SPL token account that holds the *payment* tokens (e.g. XBT, USDC).
    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = xyber_token,
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    /// The SPL mint of the payment token (e.g., USDC).
    pub payment_mint: Box<Account<'info, Mint>>,

    /// Token mint (fully minted at init).
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    /// The vault that holds project’s tokens.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = xyber_token
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    /// The user’s token account holding tokens.
    #[account(
        mut,
        has_one = mint
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// The user’s associated token account for the *payment* token.
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = payment_mint,
        associated_token::authority = user
    )]
    pub user_payment_account: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

pub fn sell_exact_input_instruction(ctx: Context<SellToken>, user_token_amount: u64) -> Result<()> {
    // 0) Prevent sells if the token is already graduated (assets locked).
    require!(
        !ctx.accounts.xyber_token.is_graduated,
        CustomError::TokenIsGraduated
    );

    // 1) Scale the user token amount by the mint decimals.
    let decimal_factor = ctx.accounts.mint.decimals as u32;
    let tokens_to_transfer = user_token_amount
        .checked_mul(10_u64.pow(decimal_factor))
        .ok_or(CustomError::MathOverflow)?;

    // 2) Transfer tokens from the user to the vault.
    let user_to_vault_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(user_to_vault_ctx, tokens_to_transfer)?;

    // 3) Calculate how many payment (base) tokens the user should receive.
    let base_token_amount = ctx
        .accounts
        .xyber_core
        .bonding_curve
        .sell_exact_input(user_token_amount)?;

    // Ensure the escrow holds enough base tokens.
    require!(
        base_token_amount <= ctx.accounts.escrow_token_account.amount,
        CustomError::InsufficientEscrowBalance
    );

    // 4) Transfer base tokens from escrow to the user using the PDA signature.
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let xyber_token_bump = ctx.bumps.xyber_token;

    let seeds: [&[u8]; 4] = [
        b"xyber_token",
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[xyber_token_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let escrow_to_user_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.user_payment_account.to_account_info(),
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(escrow_to_user_ctx, base_token_amount)?;

    Ok(())
}

pub fn sell_exact_output_instruction(
    ctx: Context<SellToken>,
    base_tokens_requested: u64,
) -> Result<()> {
    // 0) Prevent sells if the token is already graduated.
    require!(
        !ctx.accounts.xyber_token.is_graduated,
        CustomError::TokenIsGraduated
    );

    // 1) Calculate how many user tokens are required to receive the requested base tokens.
    let user_tokens_required = ctx
        .accounts
        .xyber_core
        .bonding_curve
        .sell_exact_output(base_tokens_requested)?;

    let decimal_factor = ctx.accounts.mint.decimals as u32;
    let user_tokens_required_scaled = user_tokens_required
        .checked_mul(10_u64.pow(decimal_factor))
        .ok_or(CustomError::MathOverflow)?;

    // 2) Transfer the required user tokens from the user to the vault.
    let user_to_vault_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(user_to_vault_ctx, user_tokens_required_scaled)?;

    // 3) Verify that the escrow holds enough base tokens.
    require!(
        base_tokens_requested <= ctx.accounts.escrow_token_account.amount,
        CustomError::InsufficientEscrowBalance
    );

    // 4) Transfer the requested base tokens from escrow to the user.
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let xyber_token_bump = ctx.bumps.xyber_token;

    let seeds: [&[u8]; 4] = [
        b"xyber_token",
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[xyber_token_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let escrow_to_user_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.user_payment_account.to_account_info(),
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(escrow_to_user_ctx, base_tokens_requested)?;

    Ok(())
}
