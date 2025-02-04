use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::{xyber_params, XyberToken};

#[derive(Accounts)]
pub struct BuyToken<'info> {
    /// CHECK: ...
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: creator account
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Box<Account<'info, XyberToken>>,

    // Box large token accounts
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
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}
/// (1) Buy tokens by specifying the amount of payment tokens (buy_exact_input).
pub fn buy_exact_input_instruction(ctx: Context<BuyToken>, payment_amount: u64) -> Result<()> {
    // 1) Use the bonding curve to determine the number of tokens to mint from the given payment amount.
    let tokens_u128 = {
        let xyber_token = &mut ctx.accounts.xyber_token;
        let tokens_u128 = xyber_token.bonding_curve.buy_exact_input(payment_amount)?;
        require!(
            tokens_u128 as u64 <= xyber_token.supply,
            CustomError::InsufficientTokenSupply
        );
        tokens_u128
    };

    // 2) Transfer payment tokens from the buyer's payment account to the escrow token account.
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_payment_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, payment_amount)?;

    // 3) Mint tokens to the buyer.
    let bump = ctx.bumps.xyber_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let signer_seeds = &[
        b"xyber_token".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    let raw_tokens_u64 = tokens_u128 as u64;
    let minted_tokens_u64 = raw_tokens_u64 * 10_u64.pow(xyber_params::DECIMALS as u32);

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.xyber_token.to_account_info(),
            },
            &[signer_seeds],
        ),
        minted_tokens_u64,
    )?;

    // 4) Update the total token supply.
    let xyber_token = &mut ctx.accounts.xyber_token;
    xyber_token.supply = xyber_token
        .supply
        .checked_sub(raw_tokens_u64)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

/// (2) Buy tokens by specifying the exact number of tokens desired (buy_exact_output).
///     This function calculates the required amount of payment tokens to purchase `tokens_out`.
pub fn buy_exact_output_instruction(
    ctx: Context<BuyToken>, // Boxed as well
    tokens_out: u64,
) -> Result<()> {
    // 1) Verify that the token supply is sufficient for the requested amount.
    {
        let xyber_token = &mut ctx.accounts.xyber_token;
        require!(
            tokens_out <= xyber_token.supply,
            CustomError::InsufficientTokenSupply
        );
    }

    // 2) Calculate the required payment tokens using bonding_curve.buy_exact_output.
    let payment_required = {
        let xyber_token = &mut ctx.accounts.xyber_token;
        xyber_token.bonding_curve.buy_exact_output(tokens_out)?
    };

    // 3) Transfer payment tokens from the buyer's payment account to the escrow token account.
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_payment_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, payment_required)?;

    // 4) Mint exactly `tokens_out` tokens (accounting for decimals).
    let bump = ctx.bumps.xyber_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let signer_seeds = &[
        b"xyber_token".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    let minted_tokens_u64 = tokens_out
        .checked_mul(10_u64.pow(xyber_params::DECIMALS as u32))
        .ok_or(CustomError::MathOverflow)?;

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.xyber_token.to_account_info(),
            },
            &[signer_seeds],
        ),
        minted_tokens_u64,
    )?;

    // 5) Update the total token supply.
    let xyber_token = &mut ctx.accounts.xyber_token;
    xyber_token.supply = xyber_token
        .supply
        .checked_sub(tokens_out)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}
