use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::system_program::Transfer;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::curves::traits::BondingCurveTrait;
use crate::errors::CustomError;
use crate::{omni_params, OwnedToken};

// ------------------------------------------------------------------------
// BuyToken
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct BuyToken<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

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
    /// CHECK: Escrow for SOL
    pub escrow_pda: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    // Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

/// (1) Buy tokens by specifying lamports (buy_exact_input).
pub fn buy_exact_input_instruction(ctx: Context<BuyToken>, lamports: u64) -> Result<()> {
    // 1) Use the bonding curve to find out how many tokens get minted from `lamports`.
    let tokens_u128 = {
        let owned_token = &mut ctx.accounts.owned_token;
        let tokens_u128 = owned_token.bonding_curve.buy_exact_input(lamports);
        require!(
            tokens_u128 as u64 <= owned_token.supply,
            CustomError::InsufficientTokenSupply
        );
        tokens_u128
    };

    // 2) Transfer lamports from buyer -> escrow
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.escrow_pda.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, lamports)?;

    // 3) Mint tokens to the buyer
    let bump = ctx.bumps.owned_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let signer_seeds = &[
        b"owned_token".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    require!(tokens_u128 <= u64::MAX as u128, CustomError::MathOverflow);
    let raw_tokens_u64 = tokens_u128 as u64;
    let minted_tokens_u64 = raw_tokens_u64 * 10_u64.pow(omni_params::DECIMALS as u32);

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.owned_token.to_account_info(),
            },
            &[signer_seeds],
        ),
        minted_tokens_u64,
    )?;

    // 4) Update token supply
    let owned_token = &mut ctx.accounts.owned_token;
    owned_token.supply = owned_token
        .supply
        .checked_sub(raw_tokens_u64)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

/// (2) Buy tokens by specifying the exact number of tokens desired (buy_exact_output).
///     This calculates how many lamports are needed to purchase `tokens_out`.
pub fn buy_exact_output_instruction(ctx: Context<BuyToken>, tokens_out: u64) -> Result<()> {
    // 1) Ensure we can fulfill the token request from the current supply
    {
        let owned_token = &mut ctx.accounts.owned_token;
        require!(
            tokens_out <= owned_token.supply,
            CustomError::InsufficientTokenSupply
        );
    }

    // 2) Compute how many lamports are required using buy_exact_output
    let lamports_required = {
        let owned_token = &mut ctx.accounts.owned_token;
        let lamports_u64 = owned_token
            .bonding_curve
            .buy_exact_output(tokens_out as u128);
        lamports_u64
    };

    // 3) Transfer those lamports from the buyer -> escrow
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.escrow_pda.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, lamports_required)?;

    // 4) Mint exactly `tokens_out` raw tokens (convert with decimals)
    let bump = ctx.bumps.owned_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let signer_seeds = &[
        b"owned_token".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    let minted_tokens_u64 = tokens_out
        .checked_mul(10_u64.pow(omni_params::DECIMALS as u32))
        .ok_or(CustomError::MathOverflow)?;

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.owned_token.to_account_info(),
            },
            &[signer_seeds],
        ),
        minted_tokens_u64,
    )?;

    // 5) Update token supply
    let owned_token = &mut ctx.accounts.owned_token;
    owned_token.supply = owned_token
        .supply
        .checked_sub(tokens_out)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}
