use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::events::GraduationTriggered;
use crate::events::XyberInstructionType;
use crate::events::XyberSwapEvent;
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

    let escrow_balance = ctx.accounts.escrow_token_account.amount;

    // 1) Determine the token amount for `payment_amount`.
    let (actual_tokens_out, _new_x) = ctx
        .accounts
        .xyber_core
        .bonding_curve
        .buy_exact_input(escrow_balance, payment_amount)?;

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
    let updated_escrow_balance = escrow_balance
        .checked_add(payment_amount)
        .ok_or(CustomError::MathOverflow)?;

    let real_escrow_tokens =
        updated_escrow_balance / 10_u64.pow(ctx.accounts.payment_mint.decimals as u32);

    let grad_threshold = effective_threshold_for_chains(
        ctx.accounts.xyber_core.grad_threshold,
        ctx.accounts.xyber_token.total_chains,
    )?;

    if real_escrow_tokens >= grad_threshold {
        ctx.accounts.xyber_token.is_graduated = true;
        emit!(GraduationTriggered {
            buyer: ctx.accounts.buyer.key(),
            escrow_balance: ctx.accounts.escrow_token_account.amount,
            vault: ctx.accounts.vault_token_account.key(),
            creator: ctx.accounts.xyber_token.creator.key(),
            escrow: ctx.accounts.escrow_token_account.key(),
            token_seed: ctx.accounts.token_seed.key(),
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

    emit!(XyberSwapEvent {
        ix_type: XyberInstructionType::BuyExactIn,
        token_seed: ctx.accounts.token_seed.key(),
        user: ctx.accounts.buyer.key(),
        base_amount: payment_amount,
        token_amount: actual_tokens_out,
        vault_token_amount: escrow_balance,
    });

    Ok(())
}

pub fn effective_threshold_for_chains(
    base_threshold: u64,
    chain_count: u8,
) -> std::result::Result<u64, CustomError> {
    if chain_count <= 1 {
        return Ok(base_threshold);
    }

    let extra_chains = chain_count.saturating_sub(1);
    let total_percent = 100_u64
        .checked_add(
            25_u64
                .checked_mul(extra_chains as u64)
                .ok_or(CustomError::MathOverflow)?,
        )
        .ok_or(CustomError::MathOverflow)?;

    let new_threshold = base_threshold
        .checked_mul(total_percent)
        .ok_or(CustomError::MathOverflow)?
        .checked_div(100)
        .ok_or(CustomError::MathOverflow)?;

    Ok(new_threshold)
}
