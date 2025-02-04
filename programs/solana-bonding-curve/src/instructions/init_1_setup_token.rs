use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, Mint, MintTo, SetAuthority, Token, TokenAccount,
};

use crate::xyber_params;
use crate::XyberToken;

#[derive(Accounts)]
pub struct InitAndMint<'info> {
    // Re-derive the PDA using the same seeds.
    #[account(
        mut,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub xyber_token: Account<'info, XyberToken>,

    /// CHECK: Used solely as a seed for PDA derivation.
    pub token_seed: AccountInfo<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        mint::decimals = xyber_params::DECIMALS,
        mint::authority = xyber_token
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = xyber_token
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Box<Account<'info, TokenAccount>>,

    // Program accounts (small, so unboxed is fine)
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn init_and_mint_full_supply_instruction(ctx: Context<InitAndMint>) -> Result<()> {
    // 1. Retrieve the total supply from the core state (a_total_tokens) stored earlier.
    let total_supply = ctx.accounts.xyber_token.bonding_curve.a_total_tokens;

    // 2. Update the core state with the addresses of the heavy accounts.
    {
        let token = &mut ctx.accounts.xyber_token;
        token.mint = ctx.accounts.mint.key();
        token.vault = ctx.accounts.vault_token_account.key();
    }

    // 3. Retrieve the bump and construct the seeds array.
    let bump = ctx.bumps.xyber_token;
    let seeds = &[
        b"xyber_token".as_ref(),
        ctx.accounts.creator.key.as_ref(),
        ctx.accounts.token_seed.key.as_ref(),
        &[bump],
    ];

    // 4. Mint the total supply into the vault token account using the PDA as mint authority.
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.xyber_token.to_account_info(),
            },
            &[seeds],
        ),
        total_supply,
    )?;

    // 5. Remove the mint authority (set it to None) so no further tokens can be minted.
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
