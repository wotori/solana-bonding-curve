use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::XyberToken;

#[derive(Accounts)]
#[instruction(deposit_amount: u64)]
pub struct MintInitialTokens<'info> {
    /// CHECK:
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Box<Account<'info, XyberToken>>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = payment_mint,
        associated_token::authority = xyber_token,
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    // Payment token mint
    pub payment_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = creator,
    )]
    pub creator_payment_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = xyber_token
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    /// CHECK: associated_token_program
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn mint_initial_tokens_instruction(
    ctx: Context<MintInitialTokens>,
    deposit_amount: u64,
) -> Result<()> {
    // 1) Transfer payment tokens from the creator -> escrow
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.creator_payment_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.creator.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, deposit_amount)?;
    msg!("DEBUG: Payment transfer SUCCESS");

    // 2) Calculate how many project tokens the creator receives
    msg!("DEBUG: Calling buy_exact_input() in the bonding curve...");
    let tokens_out_u128 = ctx
        .accounts
        .xyber_token
        .bonding_curve
        .buy_exact_input(deposit_amount)?;
    msg!(
        "DEBUG: buy_exact_input returned tokens_out_u128={}",
        tokens_out_u128
    );

    // (Optional) Check vault balance
    require!(
        tokens_out_u128 <= ctx.accounts.vault_token_account.amount,
        CustomError::InsufficientTokenVaultBalance
    );

    // 3) Transfer tokens from vault -> creator
    let decimals = ctx.accounts.mint.decimals as u32;
    let tokens_out_scaled = (tokens_out_u128 as u64)
        .checked_mul(10_u64.pow(decimals))
        .ok_or(CustomError::MathOverflow)?;

    // Store the keys/bump in local variables
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let xyber_token_bump = ctx.bumps.xyber_token;

    // Create the PDA seeds array, then wrap it in a slice
    let seeds: [&[u8]; 4] = [
        b"xyber_token",
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[xyber_token_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_tokens_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.creator_token_account.to_account_info(),
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_tokens_ctx, tokens_out_scaled)?;
    msg!("DEBUG: Token transfer SUCCESS!");

    Ok(())
}
