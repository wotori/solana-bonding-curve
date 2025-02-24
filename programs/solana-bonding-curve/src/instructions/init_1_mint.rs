use crate::errors::CustomError;
use crate::xyber_params;
use crate::xyber_params::TokenParams;
use crate::XyberCore;
use crate::XyberToken;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use token_factory::cpi;
use token_factory::cpi::accounts::CreateAndMintToken;

#[derive(Accounts)]
pub struct InitAndMint<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"xyber_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        space = XyberToken::LEN
    )]
    pub xyber_token: Account<'info, XyberToken>,

    #[account(
        mut,
        seeds = [b"xyber_core"],
        bump
    )]
    pub xyber_core: Account<'info, XyberCore>,

    /// CHECK: 32 bytes used for PDA derivation
    pub token_seed: AccountInfo<'info>,

    /// CHECK: Minted by the factory
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: Factory-created ATA for minted tokens
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,

    /// CHECK: Metadata account created by the factory
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_metadata_program: Program<'info, anchor_spl::metadata::Metadata>,
    pub token_program: Program<'info, Token>,

    /// CHECK: Verified via address constraint
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// External factory program
    pub token_factory_program: Program<'info, token_factory::program::TokenFactory>,
}

pub fn mint_full_supply_instruction(ctx: Context<InitAndMint>, params: TokenParams) -> Result<()> {
    let total_supply = ctx.accounts.xyber_core.bonding_curve.a_total_tokens;

    let token_seed_vec = ctx.accounts.token_seed.key().to_bytes().to_vec();
    require_eq!(token_seed_vec.len(), 32, CustomError::InvalidSeed);

    let cpi_accounts = CreateAndMintToken {
        payer: ctx.accounts.creator.to_account_info(),
        vault_owner: ctx.accounts.xyber_token.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        token_vault: ctx.accounts.vault_token_account.to_account_info(),
        metadata_account: ctx.accounts.metadata_account.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
        token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_factory_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    let raw_total_supply = total_supply * 10u64.pow(xyber_params::DECIMALS as u32); // TODO: pass from XyberToken states
    cpi::create_and_mint_token(
        cpi_ctx,
        token_seed_vec,
        xyber_params::DECIMALS,
        raw_total_supply,
        params.name,
        params.symbol,
        params.uri,
    )?;

    let xyber_token = &mut ctx.accounts.xyber_token;

    xyber_token.mint = ctx.accounts.mint.key();
    xyber_token.vault = ctx.accounts.vault_token_account.key();
    xyber_token.creator = ctx.accounts.creator.key();

    Ok(())
}
