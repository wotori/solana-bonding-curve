use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::mpl_token_metadata::{
    instructions::{CreateV1, CreateV1InstructionArgs},
    types::TokenStandard,
    ID as MPL_METADATA_ID,
};
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

#[account]
pub struct OwnedToken {
    pub supply: u64, // Tracks total supply
}

impl OwnedToken {
    pub const LEN: usize = 8 + 8; // Discriminator + supply
}

declare_id!("Da4dAJgYgs6Z4pcWWZzzvpdprtUB9hUDvoHkyJpQNYBz");

#[program]
pub mod bonding_curve {
    use super::*;
    use anchor_lang::solana_program::program::invoke_signed;

    pub fn create_token_instruction(
        ctx: Context<CreateToken>,
        total_supply: u64,
        initial_mint_amount: u64,
    ) -> Result<()> {
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.supply = total_supply;

        let creator_key = ctx.accounts.creator.key();
        let token_seed_key = ctx.accounts.token_seed.key();
        let bump = ctx.bumps.owned_token;

        let signer_seeds = &[
            b"owned_token".as_ref(),
            creator_key.as_ref(),
            token_seed_key.as_ref(),
            &[bump],
        ];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.creator_token_account.to_account_info(),
                    authority: ctx.accounts.owned_token.to_account_info(),
                },
                &[signer_seeds],
            ),
            initial_mint_amount,
        )?;

        Ok(())
    }

    pub fn set_metadata_instruction(
        ctx: Context<SetMetadata>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let create_v1 = CreateV1 {
            metadata: ctx.accounts.metadata.key(),
            master_edition: None,
            mint: (ctx.accounts.mint.key(), false),
            authority: ctx.accounts.owned_token.key(),
            payer: ctx.accounts.creator.key(),
            update_authority: (ctx.accounts.creator.key(), true),
            system_program: system_program::ID,
            sysvar_instructions: ctx.accounts.sysvar_instructions.key(),
            spl_token_program: ctx.accounts.token_program.key(),
        };

        let args = CreateV1InstructionArgs {
            name,
            symbol,
            uri,
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

        let ix = create_v1.instruction(args);

        let creator_key = ctx.accounts.creator.key();
        let token_seed_key = ctx.accounts.token_seed.key();
        let bump = ctx.bumps.owned_token;

        let signer_seeds = &[
            b"owned_token".as_ref(),
            creator_key.as_ref(),
            token_seed_key.as_ref(),
            &[bump],
        ];

        invoke_signed(
            &ix,
            &[
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.owned_token.to_account_info(),
                ctx.accounts.creator.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.sysvar_instructions.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(total_supply: u64, initial_mint_amount: u64)]
pub struct CreateToken<'info> {
    #[account()]
    /// CHECK: Used only as a seed for PDA
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        payer = creator,
        space = OwnedToken::LEN
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = owned_token
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SetMetadata<'info> {
    #[account()]
    /// CHECK: Must match the seed used in CreateToken
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(mut)]
    /// CHECK: The mint
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Metadata PDA
    pub metadata: UncheckedAccount<'info>,

    #[account(address = MPL_METADATA_ID)]
    /// CHECK: Metaplex program
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,

    #[account(address = anchor_spl::token::ID)]
    /// CHECK: SPL Token Program
    pub token_program: UncheckedAccount<'info>,

    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar Instructions
    pub sysvar_instructions: UncheckedAccount<'info>,
}
