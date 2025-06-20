use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::curves::BondingCurveTrait;
use crate::errors::CustomError;
use crate::events::XyberInstructionType;
use crate::events::XyberSwapEvent;
use crate::XyberCore;
use crate::XyberToken;

#[derive(Accounts)]
pub struct SellToken<'info> {
    /// CHECK: used as a seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

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

    /// The escrow SPL token account that holds the *payment* tokens (e.g. USDC).
    #[account(
        mut,
        associated_token::mint = payment_mint,
        associated_token::authority = xyber_token,
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    /// The SPL mint of the payment token (e.g., USDC).
    pub payment_mint: Box<Account<'info, Mint>>,

    /// Token mint (fully minted at init).
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    /// The vault that holds project’s tokens.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = xyber_token
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    /// The user’s token account holding tokens.
    #[account(
        mut,
        has_one = mint
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// The user’s associated token account for the *payment* token.
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = payment_mint,
        associated_token::authority = user
    )]
    pub user_payment_account: Box<Account<'info, TokenAccount>>,

    /// Agent's payment token account
    #[account(mut)]
    pub agent_payment_account: Box<Account<'info, TokenAccount>>,

    /// Treasury's payment token account
    #[account(mut)]
    pub treasury_payment_account: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

/// Sells an *exact input* of project tokens in exchange for base (payment) tokens.
/// Enforces a “minimum” base tokens out to guard against slippage.
pub fn sell_exact_input_instruction(
    ctx: Context<SellToken>,
    user_token_amount: u64,
    min_base_amount_out: u64, // slippage guard
) -> Result<()> {
    // 0) Prevent sells if the token is already graduated (assets locked).
    require!(
        !ctx.accounts.xyber_token.is_graduated,
        CustomError::TokenIsGraduated
    );

    require_keys_eq!(
        ctx.accounts.payment_mint.key(),
        ctx.accounts.xyber_core.accepted_base_mint,
        CustomError::WrongPaymentMint
    );

    // Validate agent and treasury token accounts
    let agent_wallet = ctx.accounts.xyber_token.agent_wallet_pubkey;
    let expected_agent_ata = anchor_spl::associated_token::get_associated_token_address(
        &agent_wallet,
        &ctx.accounts.payment_mint.key(),
    );
    require_keys_eq!(
        ctx.accounts.agent_payment_account.key(),
        expected_agent_ata,
        CustomError::InvalidAgentTokenAccount
    );

    let treasury_wallet = ctx.accounts.xyber_core.treasury_wallet;
    let expected_treasury_ata = anchor_spl::associated_token::get_associated_token_address(
        &treasury_wallet,
        &ctx.accounts.payment_mint.key(),
    );
    require_keys_eq!(
        ctx.accounts.treasury_payment_account.key(),
        expected_treasury_ata,
        CustomError::InvalidTreasuryTokenAccount
    );

    // 1) Scale the user token amount by the mint decimals.
    let decimal_factor = ctx.accounts.mint.decimals as u32;
    let tokens_to_transfer = user_token_amount
        .checked_mul(10_u64.pow(decimal_factor))
        .ok_or(CustomError::MathOverflow)?;

    // 2) Transfer tokens from the user to the vault.
    let user_to_vault_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(user_to_vault_ctx, tokens_to_transfer)?;

    // 3) Calculate how many base (payment) tokens the user should receive.
    let escrow_balance = ctx.accounts.escrow_token_account.amount;
    let (base_token_amount, _new_x) = ctx
        .accounts
        .xyber_core
        .bonding_curve
        .sell_exact_input(escrow_balance, user_token_amount)?;
    msg!("sell_exact_input actual_tokens_out = {}", base_token_amount);

    // 4) Enforce slippage check: base_token_amount >= min_base_amount_out
    require!(
        base_token_amount >= min_base_amount_out,
        CustomError::SlippageExceeded
    );

    // 5) Ensure the escrow holds enough base tokens.
    require!(
        base_token_amount <= ctx.accounts.escrow_token_account.amount,
        CustomError::InsufficientEscrowBalance
    );

    crate::utils::transfer_commission(
        &ctx.accounts.token_program,
        &ctx.accounts.escrow_token_account.to_account_info(),
        &ctx.accounts.agent_payment_account.to_account_info(),
        &ctx.accounts.treasury_payment_account.to_account_info(),
        &ctx.accounts.xyber_token.to_account_info(),
        &[
            b"xyber_token",
            ctx.accounts.token_seed.key().as_ref(),
            &[ctx.bumps.xyber_token],
        ],
        base_token_amount,
        ctx.accounts.xyber_core.commission_rate,
    )?;

    let commission_amount = base_token_amount
        .checked_mul(ctx.accounts.xyber_core.commission_rate)
        .ok_or(CustomError::MathOverflow)?
        .checked_div(10_000)
        .ok_or(CustomError::MathOverflow)?;

    let net_base_amount = base_token_amount
        .checked_sub(commission_amount)
        .ok_or(CustomError::MathOverflow)?;

    // 6) Transfer base tokens from escrow to the user using the PDA signature.
    let token_seed_key = ctx.accounts.token_seed.key();
    let xyber_token_bump = ctx.bumps.xyber_token;

    let seeds: [&[u8]; 3] = [b"xyber_token", token_seed_key.as_ref(), &[xyber_token_bump]];
    let signer_seeds = &[&seeds[..]];

    let escrow_to_user_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.user_payment_account.to_account_info(),
            authority: ctx.accounts.xyber_token.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(escrow_to_user_ctx, net_base_amount)?;

    emit!(XyberSwapEvent {
        ix_type: XyberInstructionType::SellExactIn,
        token_seed: ctx.accounts.token_seed.key(),
        user: ctx.accounts.user.key(),
        base_amount: net_base_amount,
        token_amount: tokens_to_transfer,
        vault_token_amount: escrow_balance,
    });

    Ok(())
}
