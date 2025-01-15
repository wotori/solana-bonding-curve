use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};

use anchor_spl::metadata::mpl_token_metadata::{instructions::CreateV1, ID as MPL_METADATA_ID};

pub mod events;
pub mod state;

use events::TokenLaunched;
use state::{OwnedToken, TokenLaunchParams};

declare_id!("Da4dAJgYgs6Z4pcWWZzzvpdprtUB9hUDvoHkyJpQNYBz");

#[program]
pub mod bonding_curve {
    use super::*;
    use anchor_lang::solana_program::program::invoke;
    use anchor_lang::solana_program::sysvar;
    use anchor_spl::metadata::mpl_token_metadata::instructions::CreateV1InstructionArgs;
    use anchor_spl::metadata::mpl_token_metadata::types::TokenStandard;

    /// Launch a new token with metadata
    pub fn launch_token(ctx: Context<LaunchToken>, params: TokenLaunchParams) -> Result<()> {
        // 1. Initialize OwnedToken
        {
            let owned_token = &mut ctx.accounts.owned_token;
            owned_token.token_name = params.token_name.clone();
            owned_token.ticker = params.ticker.clone();
            owned_token.supply = params.supply;
            owned_token.initial_buy_amount = params.initial_buy_amount;
            owned_token.initial_buy_price = params.initial_buy_price;
            owned_token.target_chains = params.target_chains.clone();
            owned_token.public_token = params.public_token;
            owned_token.bonding_curve_coefficients = params.bonding_curve_coefficients.clone();
            owned_token.metadata_uri = params.metadata_uri.clone();
        }

        // 2. Mint tokens to the owner's ATA
        {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.owned_token.to_account_info(),
                },
            );
            token::mint_to(cpi_ctx, params.initial_buy_amount)?;
        }

        // 3. Create Token Metadata via Metaplex
        {
            let owned_token = &ctx.accounts.owned_token;
            let mint_key = ctx.accounts.mint.key();

            // Derive metadata PDA
            let (metadata_pda, _bump) = Pubkey::find_program_address(
                &[b"metadata", MPL_METADATA_ID.as_ref(), mint_key.as_ref()],
                &MPL_METADATA_ID,
            );

            // Build the CreateV1 accounts struct
            let create_metadata_accounts_v1 = CreateV1 {
                metadata: metadata_pda,
                master_edition: None, // Not needed for a fungible
                mint: (mint_key, false),
                authority: ctx.accounts.owner.key(),
                payer: ctx.accounts.owner.key(),
                update_authority: (ctx.accounts.owner.key(), true),
                system_program: ctx.accounts.system_program.key(),
                sysvar_instructions: sysvar::instructions::ID,
                spl_token_program: token::ID,
            };

            // Build the instruction args
            let args = CreateV1InstructionArgs {
                name: owned_token.token_name.clone(),
                symbol: owned_token.ticker.clone(),
                uri: owned_token.metadata_uri.clone(),
                seller_fee_basis_points: 0,
                creators: None,
                primary_sale_happened: false,
                is_mutable: false,
                token_standard: TokenStandard::Fungible,
                collection: None,
                uses: None,
                collection_details: None,
                rule_set: None,
                decimals: Some(9),
                print_supply: None,
            };

            let create_md_ix = create_metadata_accounts_v1.instruction(args);

            // Invoke Metaplex metadata creation
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

        // 4. Emit TokenLaunched event
        {
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
        }

        Ok(())
    }
}

/// Context for launch_token
#[derive(Accounts)]
pub struct LaunchToken<'info> {
    /// Initialize OwnedToken account
    #[account(init, payer = owner, space = OwnedToken::LEN)]
    pub owned_token: Account<'info, OwnedToken>,

    /// Owner of the token
    #[account(mut)]
    pub owner: Signer<'info>,

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

    /// Metaplex Token Metadata program
    #[account(address = MPL_METADATA_ID)]
    /// CHECK: This is the official Metaplex Token Metadata program account.
    pub token_metadata_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
}
