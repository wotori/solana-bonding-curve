use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};

pub mod events;
pub mod state;

use events::TokenLaunched;
use state::{OwnedToken, TokenLaunchParams};

declare_id!("E48ijHDaZqdVBiGgCGGJRQTM373TsjYPGv8rpVzu4P9R");

#[program]
pub mod bonding_curve {
    use super::*;
    use anchor_lang::solana_program::program::invoke;
    use mpl_token_metadata::{instruction::create_metadata_accounts_v2, ID as MPL_METADATA_ID};

    /// Initialize bonding curve and token with selected parameters.
    /// For displaying token in wallets, use Metaplex instructions separately.
    pub fn create_owned_token(
        ctx: Context<CreateOwnedToken>,
        params: TokenLaunchParams,
    ) -> ProgramResult {
        // 1. Store minimal bonding-curve info in OwnedToken
        {
            let owned_token = &mut ctx.accounts.owned_token;
            owned_token.token_name = params.token_name.clone();
            owned_token.ticker = params.ticker.clone();
            owned_token.supply = params.supply;
            owned_token.initial_buy_amount = params.initial_buy_amount;
            owned_token.initial_buy_price = params.initial_buy_price;
            owned_token.target_chains = params.target_chains;
            owned_token.public_token = params.public_token;
            owned_token.bonding_curve_coefficients = params.bonding_curve_coefficients;
        }

        // 2. Mint a certain amount of tokens to the owner's ATA
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.owner_token_account.to_account_info(),
                authority: ctx.accounts.owned_token.to_account_info(),
            },
        );
        token::mint_to(cpi_ctx, params.initial_buy_amount)?;

        // 3.Create Metaplex Metadata (simplified example)
        // Usually pass in `name, symbol, uri` from front-end.
        if let Some(uri) = params.metadata_uri {
            let mint_key = ctx.accounts.mint.key();
            // Derive metadata PDA
            let (metadata_pda, _bump) = Pubkey::find_program_address(
                &[b"metadata", MPL_METADATA_ID.as_ref(), mint_key.as_ref()],
                &MPL_METADATA_ID,
            );

            let create_md_ix = create_metadata_accounts_v2(
                MPL_METADATA_ID,
                metadata_pda,
                mint_key,
                ctx.accounts.owner.key(),
                ctx.accounts.owner.key(),
                ctx.accounts.owner.key(),
                params.token_name,
                params.ticker,
                uri,
                None, // creators
                0,    // seller_fee_basis_points
                true, // update_authority_is_signer
                false,
            );
            invoke(
                &create_md_ix,
                &[
                    ctx.accounts.token_metadata_program.to_account_info(),
                    ctx.accounts.mint.to_account_info(),
                    ctx.accounts.owner.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        // 4. Emit event for the newly created token
        let owned_token = &ctx.accounts.owned_token;
        emit!(TokenLaunched {
            token_name: owned_token.token_name.clone(),
            ticker: owned_token.ticker.clone(),
            supply: owned_token.supply,
            initial_buy_amount: owned_token.initial_buy_amount,
            initial_buy_price: owned_token.initial_buy_price,
            target_chains: owned_token.target_chains.clone(),
            public_token: owned_token.public_token,
            bonding_curve_coefficients: owned_token.bonding_curve_coefficients.clone(),
        });

        Ok(())
    }
}

/// Context for the "create_owned_token" instruction
#[derive(Accounts)]
pub struct CreateOwnedToken<'info> {
    /// Minimal bonding-curve info
    #[account(init, payer = owner, space = OwnedToken::LEN)]
    pub owned_token: Account<'info, OwnedToken>,

    /// The owner initializing the token
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The public token liquidity pool
    #[account(mut)]
    pub owned_token_pool: Account<'info, TokenAccount>,

    /// Mint account for the new token
    #[account(
        init,
        payer = owner,
        mint::decimals = 9,
        mint::authority = owned_token
    )]
    pub mint: Account<'info, Mint>,

    /// Associated Token Account for the owner to receive minted tokens
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = owner
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    /// Metaplex Token Metadata Program (for optional metadata creation)
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Associated Token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// SPL Token program
    pub token_program: Program<'info, Token>,
}
