use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::events::GraduationTriggered;
use crate::XyberCore;
use crate::XyberToken;

#[derive(Accounts)]
pub struct BuyToken<'info> {
    /// CHECK: Used solely as a seed for PDA derivation.
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"xyber_core"],
        bump
    )]
    pub xyber_core: Account<'info, XyberCore>,

    #[account(
        mut,
        seeds = [b"xyber_token", token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Account<'info, XyberToken>,

    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = xyber_token,
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    pub payment_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = xyber_token
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = payment_mint,
        associated_token::authority = buyer
    )]
    pub buyer_payment_account: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program.
    pub system_program: UncheckedAccount<'info>,
}

pub fn buy_exact_input_instruction(
    ctx: Context<BuyToken>,
    payment_amount: u64,
    min_amount_out: u64,
) -> Result<()> {
    // 0) Reject if graduated.
    require!(
        !ctx.accounts.xyber_token.is_graduated,
        CustomError::TokenIsGraduated
    );

    require_keys_eq!(
        ctx.accounts.payment_mint.key(),
        ctx.accounts.xyber_core.accepted_base_mint,
        CustomError::WrongPaymentMint
    );

    // 1) Determine the token amount for `payment_amount`.
    let actual_tokens_out = ctx
        .accounts
        .xyber_core
        .bonding_curve
        .buy_exact_input(payment_amount)?;

    msg!(
        "buy_exact_input actual_tokens_out = {:?}",
        actual_tokens_out
    );
    msg!("Vault amount = {}", ctx.accounts.vault_token_account.amount);

    // 2) Enforce `actual_tokens_out >= min_amount_out`.
    //    (The front end will handle slippage and supply a proper `min_amount_out`.)
    require!(
        actual_tokens_out >= min_amount_out,
        CustomError::SlippageExceeded
    );

    // 3) Check vault balance.
    require!(
        actual_tokens_out <= ctx.accounts.vault_token_account.amount,
        CustomError::InsufficientTokenVaultBalance
    );

    // 4) Transfer the buyer’s payment from `buyer_payment_account` -> `escrow_token_account`.
    let transfer_payment_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_payment_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        },
    );
    token::transfer(transfer_payment_ctx, payment_amount)?;

    // 5) Check graduation condition.
    let real_escrow_tokens = ctx.accounts.escrow_token_account.amount as f64
        / 10_f64.powi(ctx.accounts.payment_mint.decimals.into());
    let price = 0.05; // TODO: read from Oracle or keep it fixed for now
    if real_escrow_tokens * price >= ctx.accounts.xyber_core.graduate_dollars_amount as f64 {
        ctx.accounts.xyber_token.is_graduated = true;
        emit!(GraduationTriggered {
            buyer: ctx.accounts.buyer.key(),
            escrow_balance: ctx.accounts.escrow_token_account.amount,
        });
    }

    // 6) Transfer `actual_tokens_out` from the vault to the buyer, accounting for decimals.
    let token_amount_with_decimals = actual_tokens_out
        .checked_mul(10_u64.pow(ctx.accounts.mint.decimals as u32))
        .ok_or(CustomError::MathOverflow)?;

    let token_seed_key = ctx.accounts.token_seed.key();
    let xyber_token_bump = ctx.bumps.xyber_token;

    let seeds: [&[u8]; 3] = [b"xyber_token", token_seed_key.as_ref(), &[xyber_token_bump]];
    let signer_seeds = &[&seeds[..]];

    let vault_transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(vault_transfer_ctx, token_amount_with_decimals)?;

    Ok(())
}

pub fn buy_exact_output_instruction(
    ctx: Context<BuyToken>,
    tokens_out: u64,
    max_payment_amount: u64,
) -> Result<()> {
    // 0) Reject if graduated.
    require!(
        !ctx.accounts.xyber_token.is_graduated,
        CustomError::TokenIsGraduated
    );

    require_keys_eq!(
        ctx.accounts.payment_mint.key(),
        ctx.accounts.xyber_core.accepted_base_mint,
        CustomError::WrongPaymentMint
    );

    // 1) Calculate how many payment tokens are needed for `tokens_out`.
    let payment_required = ctx
        .accounts
        .xyber_core
        .bonding_curve
        .buy_exact_output(tokens_out)?;

    // 2) Enforce `payment_required <= max_payment_amount`.
    require!(
        payment_required <= max_payment_amount,
        CustomError::SlippageExceeded
    );

    // 3) Check vault balance.
    require!(
        (tokens_out as u128) <= ctx.accounts.vault_token_account.amount as u128,
        CustomError::InsufficientTokenVaultBalance
    );

    // 4) Transfer payment from buyer -> escrow.
    let transfer_payment_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_payment_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        },
    );
    token::transfer(transfer_payment_ctx, payment_required)?;

    // 5) Check graduation.
    let real_escrow_tokens = ctx.accounts.escrow_token_account.amount as f64
        / 10_f64.powi(ctx.accounts.payment_mint.decimals.into());
    let price = 0.05; // TODO: read from Oracle or keep it fixed for now
    if real_escrow_tokens * price >= ctx.accounts.xyber_core.graduate_dollars_amount as f64 {
        ctx.accounts.xyber_token.is_graduated = true;
        emit!(GraduationTriggered {
            buyer: ctx.accounts.buyer.key(),
            escrow_balance: ctx.accounts.escrow_token_account.amount,
        });
    }

    // 6) Transfer tokens_out (scaled) from vault -> buyer.
    let decimals = ctx.accounts.mint.decimals as u32;
    let tokens_out_scaled = tokens_out
        .checked_mul(10_u64.pow(decimals))
        .ok_or(CustomError::MathOverflow)?;

    let token_seed_key = ctx.accounts.token_seed.key();
    let xyber_token_bump = ctx.bumps.xyber_token;

    let seeds: [&[u8]; 3] = [b"xyber_token", token_seed_key.as_ref(), &[xyber_token_bump]];
    let signer_seeds = &[&seeds[..]];

    let transfer_tokens_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_tokens_ctx, tokens_out_scaled)?;

    Ok(())
}
