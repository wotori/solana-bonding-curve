use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::system_program::Transfer;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::curves::traits::BondingCurveTrait;
use crate::errors::CustomError;
use crate::{xyber_params, XyberToken};

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
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Account<'info, XyberToken>,

    #[account(
        mut,
        seeds = [b"escrow", creator.key().as_ref(), token_seed.key().as_ref()],
        bump = xyber_token.escrow_bump,
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

/// (1) Buy tokens by specifying base_token (buy_exact_input).
pub fn buy_exact_input_instruction(ctx: Context<BuyToken>, base_token: u64) -> Result<()> {
    // 1) Use the bonding curve to find out how many tokens get minted from `base_token`.
    let tokens_u128 = {
        let xyber_token = &mut ctx.accounts.xyber_token;
        let tokens_u128 = xyber_token.bonding_curve.buy_exact_input(base_token);
        require!(
            tokens_u128 as u64 <= xyber_token.supply,
            CustomError::InsufficientTokenSupply
        );
        tokens_u128
    };

    // 2) Transfer base_token from buyer -> escrow
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.escrow_pda.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, base_token)?;

    // 3) Mint tokens to the buyer
    let bump = ctx.bumps.xyber_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let signer_seeds = &[
        b"xyber_token".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    require!(tokens_u128 <= u64::MAX, CustomError::MathOverflow);
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

    // 4) Update token supply
    let xyber_token = &mut ctx.accounts.xyber_token;
    xyber_token.supply = xyber_token
        .supply
        .checked_sub(raw_tokens_u64)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}

/// (2) Buy tokens by specifying the exact number of tokens desired (buy_exact_output).
///     This calculates how many base_token are needed to purchase `tokens_out`.
pub fn buy_exact_output_instruction(ctx: Context<BuyToken>, tokens_out: u64) -> Result<()> {
    // 1) Ensure we can fulfill the token request from the current supply
    {
        let xyber_token = &mut ctx.accounts.xyber_token;
        require!(
            tokens_out <= xyber_token.supply,
            CustomError::InsufficientTokenSupply
        );
    }

    // 2) Compute how many base_token are required using buy_exact_output
    let base_token_required = {
        let xyber_token = &mut ctx.accounts.xyber_token;
        let base_token_u64 = xyber_token.bonding_curve.buy_exact_output(tokens_out);
        base_token_u64
    };

    // 3) Transfer those base_token from the buyer -> escrow
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.escrow_pda.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, base_token_required)?;

    // 4) Mint exactly `tokens_out` raw tokens (convert with decimals)
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

    // 5) Update token supply
    let xyber_token = &mut ctx.accounts.xyber_token;
    xyber_token.supply = xyber_token
        .supply
        .checked_sub(tokens_out)
        .ok_or(CustomError::MathOverflow)?;

    Ok(())
}
