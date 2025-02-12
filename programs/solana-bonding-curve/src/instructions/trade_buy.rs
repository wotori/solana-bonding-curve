use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::events::GraduationTriggered;
use crate::XyberToken;

#[derive(Accounts)]
pub struct BuyToken<'info> {
    /// CHECK: Used solely as a seed for PDA derivation.
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Creator account.
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Box<Account<'info, XyberToken>>,

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
    expected_tokens_out: u64,
    slippage_bps: u16,
) -> Result<()> {
    // 0) Reject if graduated.
    require!(
        !ctx.accounts.xyber_token.is_graduated,
        CustomError::TokenIsGraduated
    );

    // 1) Determine the token amount for payment_amount.
    let actual_tokens_out = ctx
        .accounts
        .xyber_token
        .bonding_curve
        .buy_exact_input(payment_amount)?;

    msg!(
        "buy_exact_input actual_tokens_out = {:?}",
        actual_tokens_out
    );
    msg!("Vault amount = {}", ctx.accounts.vault_token_account.amount);

    // Only revert if actual < expected and difference exceeds slippage.
    if actual_tokens_out < expected_tokens_out {
        let diff = expected_tokens_out
            .checked_sub(actual_tokens_out)
            .ok_or(CustomError::MathOverflow)?;

        let max_allowed_diff = (expected_tokens_out as u128)
            .checked_mul(slippage_bps as u128)
            .ok_or(CustomError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(CustomError::MathOverflow)? as u64;

        require!(diff <= max_allowed_diff, CustomError::SlippageExceeded);
    }

    // Check vault balance.
    require!(
        actual_tokens_out <= ctx.accounts.vault_token_account.amount,
        CustomError::InsufficientTokenVaultBalance
    );

    // 2) Transfer buyer’s payment.
    let transfer_payment_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_payment_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        },
    );
    token::transfer(transfer_payment_ctx, payment_amount)?;

    // 2.5) Check graduation.
    let real_escrow_tokens = ctx.accounts.escrow_token_account.amount as f64
        / 10_f64.powi(ctx.accounts.payment_mint.decimals.into());
    let price = 0.05; // TODO: read from Oracle
    if real_escrow_tokens * price >= ctx.accounts.xyber_token.graduate_dollars_amount as f64 {
        ctx.accounts.xyber_token.is_graduated = true;
        emit!(GraduationTriggered {
            buyer: ctx.accounts.buyer.key(),
            escrow_balance: ctx.accounts.escrow_token_account.amount,
        });
    }


    let tokens_u64 = actual_tokens_out;
    let token_amount_with_decimals = tokens_u64
        .checked_mul(10_u64.pow(ctx.accounts.mint.decimals as u32))
        .ok_or(CustomError::MathOverflow)?;

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
    expected_payment_amount: u64,
    slippage_bps: u16,
) -> Result<()> {
    // 0) Reject if graduated.
    require!(
        !ctx.accounts.xyber_token.is_graduated,
        CustomError::TokenIsGraduated
    );

    // 1) Calculate how many payment tokens are needed for tokens_out.
    let payment_required = ctx
        .accounts
        .xyber_token
        .bonding_curve
        .buy_exact_output(tokens_out)?;

    // Only revert if payment_required > expected and difference exceeds slippage.
    if payment_required > expected_payment_amount {
        let diff = payment_required
            .checked_sub(expected_payment_amount)
            .ok_or(CustomError::MathOverflow)?;

        let max_allowed_diff = (expected_payment_amount as u128)
            .checked_mul(slippage_bps as u128)
            .ok_or(CustomError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(CustomError::MathOverflow)? as u64;

        require!(diff <= max_allowed_diff, CustomError::SlippageExceeded);
    }

    // 2) Check vault balance.
    require!(
        (tokens_out as u128) <= ctx.accounts.vault_token_account.amount as u128,
        CustomError::InsufficientTokenVaultBalance
    );

    // 3) Transfer payment from buyer -> escrow.
    let transfer_payment_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_payment_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        },
    );
    token::transfer(transfer_payment_ctx, payment_required)?;

    // 3.5) Check graduation.
    let real_escrow_tokens = ctx.accounts.escrow_token_account.amount as f64
        / 10_f64.powi(ctx.accounts.payment_mint.decimals.into());
    let price = 0.05; // TODO: read from Oracle
    if real_escrow_tokens * price >= ctx.accounts.xyber_token.graduate_dollars_amount as f64 {
        ctx.accounts.xyber_token.is_graduated = true;
        emit!(GraduationTriggered {
            buyer: ctx.accounts.buyer.key(),
            escrow_balance: ctx.accounts.escrow_token_account.amount,
        });
    }

    // 4) Transfer tokens_out (scaled) from vault -> buyer.
    let decimals = ctx.accounts.mint.decimals as u32;
    let tokens_out_scaled = tokens_out
        .checked_mul(10_u64.pow(decimals))
        .ok_or(CustomError::MathOverflow)?;

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
