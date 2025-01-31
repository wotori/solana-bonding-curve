use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::system_program;

use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::curves::traits::BondingCurveTrait;
use crate::errors::CustomError;
use crate::{omni_params, OwnedToken};

// ------------------------------------------------------------------------
// SellToken
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct SellToken<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: seed
    #[account()]
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"omni_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub omni_token: Account<'info, OwnedToken>,

    #[account(
        mut,
        seeds = [b"escrow", creator.key().as_ref(), token_seed.key().as_ref()],
        bump = omni_token.escrow_bump,
        owner = system_program::ID
    )]
    /// CHECK: escrow for SOL
    pub escrow_pda: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        has_one = mint
    )]
    pub user_token_account: Account<'info, TokenAccount>,

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
    // 1) Burn user's tokens
    // --------------------------------------------------------------------
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            from: ctx.accounts.user_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );

    let tokens_to_burn = normalized_omni_token_amount
        .checked_mul(10_u64.pow(omni_params::DECIMALS as u32))
        .ok_or(CustomError::MathOverflow)?;

    token::burn(cpi_ctx, tokens_to_burn)?;

    // --------------------------------------------------------------------
    // 2) Calculate how much SOL to return
    // --------------------------------------------------------------------
    let omni_token = &mut ctx.accounts.omni_token;
    let base_token = omni_token
        .bonding_curve
        .sell_exact_input(normalized_omni_token_amount);

    // --------------------------------------------------------------------
    // 3) Transfer SOL from escrow -> user
    // --------------------------------------------------------------------
    let bump = omni_token.escrow_bump;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();

    let escrow_seeds = &[
        b"escrow".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.escrow_pda.key(),
        &ctx.accounts.user.key(),
        base_token,
    );
    invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.escrow_pda.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[escrow_seeds],
    )?;

    // --------------------------------------------------------------------
    // 4) Increase supply
    // --------------------------------------------------------------------
    omni_token.supply = omni_token
        .supply
        .checked_add(normalized_omni_token_amount)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

pub fn sell_exact_output_instruction(
    ctx: Context<SellToken>,
    base_token_requested: u64,
) -> Result<()> {
    // --------------------------------------------------------------------
    // 1) Determine how many tokens need to be burned
    // --------------------------------------------------------------------
    let omni_token = &mut ctx.accounts.omni_token;
    let tokens_to_burn = omni_token
        .bonding_curve
        .sell_exact_output(base_token_requested);

    // Convert to raw token amount with decimals
    let raw_burn_amount = tokens_to_burn
        .checked_mul(10u64.pow(omni_params::DECIMALS as u32))
        .ok_or(CustomError::MathOverflow)?;

    // --------------------------------------------------------------------
    // 2) Burn that many tokens from the user's token account
    // --------------------------------------------------------------------
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Burn {
            from: ctx.accounts.user_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );

    token::burn(cpi_ctx, raw_burn_amount)?;

    // --------------------------------------------------------------------
    // 3) Transfer the exact lamports to the user
    // --------------------------------------------------------------------
    let bump = omni_token.escrow_bump;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();

    let escrow_seeds = &[
        b"escrow".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.escrow_pda.key(),
        &ctx.accounts.user.key(),
        base_token_requested,
    );

    invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.escrow_pda.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[escrow_seeds],
    )?;

    // --------------------------------------------------------------------
    // 4) Increase the supply on the OwnedToken
    // --------------------------------------------------------------------
    omni_token.supply = omni_token
        .supply
        .checked_add(tokens_to_burn as u64)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}
