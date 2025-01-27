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
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(
        mut,
        seeds = [b"escrow", creator.key().as_ref(), token_seed.key().as_ref()],
        bump = owned_token.escrow_bump,
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

pub fn sell_instruction(ctx: Context<SellToken>, normalized_token_amount: u64) -> Result<()> {
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

    let tokens_to_burn = normalized_token_amount
        .checked_mul(10_u64.pow(omni_params::DECIMALS as u32))
        .ok_or(CustomError::MathOverflow)?;

    token::burn(cpi_ctx, tokens_to_burn)?;

    // --------------------------------------------------------------------
    // 2) Calculate how much SOL to return
    // --------------------------------------------------------------------
    let owned_token = &mut ctx.accounts.owned_token;
    let lamports_return = owned_token
        .bonding_curve
        .sell_exact_input(normalized_token_amount as u128);

    // --------------------------------------------------------------------
    // 3) Transfer SOL from escrow -> user
    // --------------------------------------------------------------------
    let bump = owned_token.escrow_bump;
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
        lamports_return,
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
    owned_token.supply = owned_token
        .supply
        .checked_add(normalized_token_amount)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}
