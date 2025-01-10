use anchor_lang::prelude::*;

pub mod events;
pub mod state;

use anchor_spl::token::{Mint, Token, TokenAccount};
use state::{OwnedToken, TokenLaunchParams};

declare_id!("E48ijHDaZqdVBiGgCGGJRQTM373TsjYPGv8rpVzu4P9R");

#[program]
pub mod bonding_curve {
    use anchor_lang::solana_program::entrypoint::ProgramResult;
    use events::TokenLaunched;

    use super::*;

    pub fn create_owned_token(
        ctx: Context<CreateOwnedToken>,
        params: TokenLaunchParams,
    ) -> ProgramResult {
        // Check owner's balance
        // TODO: "Check if the transaction has sufficient balance to cover deployment fees";

        // Initialize the OwnedToken account with the provided parameters
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.token_name = params.token_name;
        owned_token.ticker = params.ticker;
        owned_token.media_url = params.media_url;
        owned_token.description = params.description;
        owned_token.website = params.website;
        owned_token.twitter = params.twitter;
        owned_token.telegram = params.telegram;
        owned_token.supply = params.supply;
        owned_token.initial_buy_amount = params.initial_buy_amount;
        owned_token.initial_buy_price = params.initial_buy_price;
        owned_token.target_chains = params.target_chains;
        owned_token.public_token = params.public_token;
        owned_token.bonding_curve_coefficients = params.bonding_curve_coefficients;

        // Transfer public tokens to the token pool
        // TODO: Transfer public_token (Solana) from the owner to the owned_token_pool

        // Mint the initial amount of owned tokens to the owner
        // TODO: Mint initial_buy_amount of owned_token to the owner's wallet

        // Emit the TokenLaunched event
        emit!(TokenLaunched {
            token_name: owned_token.token_name.clone(),
            ticker: owned_token.ticker.clone(),
            media_url: owned_token.media_url.clone(),
            description: owned_token.description.clone(),
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

/// Context for the token creation instruction
#[derive(Accounts)]
pub struct CreateOwnedToken<'info> {
    /// Account to store information about the token
    #[account(init, payer = owner, space = OwnedToken::LEN)]
    pub owned_token: Account<'info, OwnedToken>,

    /// The owner initializing the token
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The public token pool
    #[account(mut)]
    pub owned_token_pool: Account<'info, TokenAccount>,

    /// Mint account for the new token
    #[account(init, payer = owner, mint::decimals = 9, mint::authority = owned_token)]
    pub mint: Account<'info, Mint>,

    /// System program
    pub system_program: Program<'info, System>,

    /// SPL Token program
    pub token_program: Program<'info, Token>,
}
