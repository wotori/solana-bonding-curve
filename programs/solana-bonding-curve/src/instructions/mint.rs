use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::system_program::Transfer;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

use crate::curves::traits::BondingCurveTrait;
use crate::errors::CustomError;
use crate::xyber_params;
use crate::XyberToken;

// ------------------------------------------------------------------------
// (3) MintInitialTokens
// ------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(deposit_lamports: u64)]
pub struct MintInitialTokens<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

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

    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,

    // Programs
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

pub fn mint_initial_tokens_instruction(
    ctx: Context<MintInitialTokens>,
    deposit_lamports: u64,
) -> Result<()> {
    // --------------------------------------------------------------------
    // 1) Transfer lamports from `creator` to Escrow PDA
    // --------------------------------------------------------------------
    msg!(
        "DEBUG: Starting mint_initial_tokens_instruction. deposit_lamports={}",
        deposit_lamports
    );
    msg!(
        "DEBUG: Escrow PDA balance BEFORE transfer: {} lamports",
        ctx.accounts.escrow_pda.to_account_info().lamports()
    );
    msg!(
        "DEBUG: Creator balance BEFORE transfer: {} lamports",
        ctx.accounts.creator.to_account_info().lamports()
    );

    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.creator.to_account_info(),
            to: ctx.accounts.escrow_pda.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, deposit_lamports)?;

    msg!(
        "DEBUG: Transfer SUCCESS. Escrow PDA balance AFTER transfer: {} lamports",
        ctx.accounts.escrow_pda.to_account_info().lamports()
    );
    msg!(
        "DEBUG: Creator balance AFTER transfer: {} lamports",
        ctx.accounts.creator.to_account_info().lamports()
    );

    // --------------------------------------------------------------------
    // 2) Calculate the token amount via the bonding curve
    // --------------------------------------------------------------------
    msg!("DEBUG: Calling buy_exact_input() in the bonding curve...");
    let minted_tokens_u128 = ctx
        .accounts
        .xyber_token
        .bonding_curve
        .buy_exact_input(deposit_lamports);

    msg!(
        "DEBUG: buy_exact_input returned minted_tokens_u128={}",
        minted_tokens_u128
    );

    require!(minted_tokens_u128 <= u64::MAX, CustomError::MathOverflow);

    let human_readable_tokens = minted_tokens_u128 as u64;
    msg!(
        "DEBUG: minted_tokens_u64={} (will pass this to token::mint_to)",
        human_readable_tokens
    );

    // --------------------------------------------------------------------
    // 3) Mint these tokens to the creator's ATA
    // --------------------------------------------------------------------
    let bump = ctx.bumps.xyber_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();

    msg!("DEBUG: Bump = {}", bump);
    msg!("DEBUG: Creator Pubkey = {}", creator_key);
    msg!("DEBUG: Token Seed Pubkey = {}", token_seed_key);

    let signer_seeds = &[
        b"xyber_token".as_ref(),
        creator_key.as_ref(),
        token_seed_key.as_ref(),
        &[bump],
    ];

    let minted_tokens_u64 = human_readable_tokens * 10_u64.pow(xyber_params::DECIMALS as u32);
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.creator_token_account.to_account_info(),
                authority: ctx.accounts.xyber_token.to_account_info(),
            },
            &[signer_seeds],
        ),
        minted_tokens_u64,
    )?;
    msg!("DEBUG: mint_to SUCCESS!");

    // --------------------------------------------------------------------
    // 4) Reduce supply
    // --------------------------------------------------------------------
    let xyber_token = &mut ctx.accounts.xyber_token;
    xyber_token.supply = xyber_token
        .supply
        .checked_sub(human_readable_tokens)
        .ok_or(CustomError::MathOverflow)?;
    msg!("DEBUG: xyber_token.supply AFTER sub={}", xyber_token.supply);

    msg!("DEBUG: Instruction complete. Returning Ok(()).");
    Ok(())
}
