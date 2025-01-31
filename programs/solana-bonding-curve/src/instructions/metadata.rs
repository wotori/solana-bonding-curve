use anchor_lang::{prelude::*, solana_program::program::invoke_signed, system_program};

use anchor_spl::metadata::mpl_token_metadata::{
    self,
    instructions::{CreateV1, CreateV1InstructionArgs},
    types::TokenStandard,
};

use crate::{omni_params, OwnedToken};

// ------------------------------------------------------------------------
// SetMetadata
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct SetMetadata<'info> {
    /// CHECK: arbitrary seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"omni_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub omni_token: Account<'info, OwnedToken>,

    /// CHECK: The mint
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: Metadata PDA
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(address = mpl_token_metadata::ID)]
    /// CHECK: Metaplex
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(address = anchor_spl::token::ID)]
    /// CHECK: SPL Token
    pub token_program: UncheckedAccount<'info>,

    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar Instructions
    pub sysvar_instructions: UncheckedAccount<'info>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
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
        authority: ctx.accounts.omni_token.key(),
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
        decimals: Some(omni_params::DECIMALS),
        print_supply: None,
    };

    let ix = create_v1.instruction(args);

    let bump = ctx.bumps.omni_token;
    let creator_key = ctx.accounts.creator.key();
    let token_seed_key = ctx.accounts.token_seed.key();
    let signer_seeds = &[
        b"omni_token".as_ref(),
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
            ctx.accounts.omni_token.to_account_info(),
            ctx.accounts.creator.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        &[signer_seeds],
    )?;

    Ok(())
}
