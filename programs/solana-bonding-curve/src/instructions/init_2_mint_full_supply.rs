use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, Mint, MintTo, SetAuthority, Token, TokenAccount,
};

use crate::XyberToken;

#[derive(Accounts)]
pub struct MintFullSupply<'info> {
    // Re-derive the PDA with the same seeds:
    // [b"xyber_token", creator.key, token_seed.key]
    #[account(
        mut,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Account<'info, XyberToken>,

    /// CHECK: Used solely for PDA derivation.
    pub token_seed: UncheckedAccount<'info>,

    // The creator is needed to supply their key as part of the seeds.
    pub creator: Signer<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn mint_full_supply_instruction(ctx: Context<MintFullSupply>, total_supply: u64) -> Result<()> {
    // Retrieve the bump from the context.
    let bump = ctx.bumps.xyber_token;

    // Reconstruct the exact seed array used in the initialization.
    let seeds = &[
        b"xyber_token".as_ref(),
        ctx.accounts.creator.key.as_ref(),
        ctx.accounts.token_seed.key.as_ref(),
        &[bump],
    ];

    // 1. Mint the full supply to the vault account using the PDA as mint authority.
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                // The PDA (xyber_token) acts as the authority.
                authority: ctx.accounts.xyber_token.to_account_info(),
            },
            &[seeds],
        ),
        total_supply,
    )?;

    // 2. Remove the mint authority so no one can mint additional tokens.
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: ctx.accounts.mint.to_account_info(),
                current_authority: ctx.accounts.xyber_token.to_account_info(),
            },
            &[seeds],
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    Ok(())
}
