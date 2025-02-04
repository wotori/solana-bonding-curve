use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::{xyber_params, XyberToken};

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
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Account<'info, XyberToken>,

    /// The escrow SPL token account that holds the payment tokens.
    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = xyber_token,
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    /// The SPL mint of the payment token.
    pub payment_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    /// The user’s token account holding XYBER tokens (to be burned).
    #[account(
        mut,
        has_one = mint
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// The user’s associated token account for the payment token.
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

pub fn sell_exact_input_instruction(
    ctx: Context<SellToken>,
    normalized_omni_token_amount: u64,
) -> Result<()> {
    // --------------------------------------------------------------------
    // 1) Burn user's XYBER tokens
    // --------------------------------------------------------------------
    {
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );

        let tokens_to_burn = normalized_omni_token_amount
            .checked_mul(10_u64.pow(xyber_params::DECIMALS as u32))
            .ok_or(CustomError::MathOverflow)?;
        token::burn(burn_ctx, tokens_to_burn)?;
    }

    // --------------------------------------------------------------------
    // 2) Calculate the amount of payment tokens to return
    // --------------------------------------------------------------------
    let base_token = {
        // Limit the mutable borrow to this block.
        let xyber_token = &mut ctx.accounts.xyber_token;
        xyber_token
            .bonding_curve
            .sell_exact_input(normalized_omni_token_amount)?
    };

    // --------------------------------------------------------------------
    // 3) Transfer payment tokens from escrow -> user
    // --------------------------------------------------------------------
    // First, get an immutable copy of the PDA's account info.
    let xyber_token_info = ctx.accounts.xyber_token.to_account_info();
    let bump = ctx.bumps.xyber_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();

    // Bind the seeds array to a local variable so it lives long enough.
    let seeds: &[&[u8]] = &[
        b"xyber_token",
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[seeds]; // This creates a slice of slices

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.user_payment_account.to_account_info(),
            authority: xyber_token_info,
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, base_token)?;

    // --------------------------------------------------------------------
    // 4) Increase token supply (adding back the sold tokens)
    // --------------------------------------------------------------------
    {
        let xyber_token = &mut ctx.accounts.xyber_token;
        xyber_token.supply = xyber_token
            .supply
            .checked_add(normalized_omni_token_amount)
            .ok_or(CustomError::MathOverflow)?;
    }

    Ok(())
}

pub fn sell_exact_output_instruction(
    ctx: Context<SellToken>,
    base_token_requested: u64,
) -> Result<()> {
    // --------------------------------------------------------------------
    // 1) Determine how many XYBER tokens need to be burned for the requested payout
    // --------------------------------------------------------------------
    let tokens_to_burn = {
        let xyber_token = &mut ctx.accounts.xyber_token;
        xyber_token
            .bonding_curve
            .sell_exact_output(base_token_requested)?
    };

    let raw_burn_amount = tokens_to_burn
        .checked_mul(10_u64.pow(xyber_params::DECIMALS as u32))
        .ok_or(CustomError::MathOverflow)?;

    // --------------------------------------------------------------------
    // 2) Burn the calculated amount of XYBER tokens from the user's token account
    // --------------------------------------------------------------------
    {
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::burn(burn_ctx, raw_burn_amount)?;
    }

    // --------------------------------------------------------------------
    // 3) Transfer the exact requested payment tokens from escrow -> user
    // --------------------------------------------------------------------
    let xyber_token_info = ctx.accounts.xyber_token.to_account_info();
    let bump = ctx.bumps.xyber_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();

    let seeds: &[&[u8]] = &[
        b"xyber_token",
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[seeds];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.user_payment_account.to_account_info(),
            authority: xyber_token_info,
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, base_token_requested)?;

    // --------------------------------------------------------------------
    // 4) Increase token supply (adding back the burned tokens)
    // --------------------------------------------------------------------
    {
        let xyber_token = &mut ctx.accounts.xyber_token;
        xyber_token.supply = xyber_token
            .supply
            .checked_add(tokens_to_burn)
            .ok_or(CustomError::MathOverflow)?;
    }

    Ok(())
}
