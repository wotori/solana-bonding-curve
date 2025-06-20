use crate::errors::CustomError;
use crate::xyber_params;
use crate::xyber_params::TokenParams;
use crate::XyberCore;
use crate::XyberToken;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use token_factory::cpi;
use token_factory::cpi::accounts::CreateAndMintToken;

#[derive(Accounts)]
#[instruction(params: TokenParams)]
pub struct InitAndMint<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"xyber_token", params.token_seed.as_ref()],
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

    /// CHECK: Minted by the factory (Not yet initialised)
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    /// CHECK: Factory-created ATA for minted tokens (Not yet initialised)
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = payment_mint,
        associated_token::authority = xyber_token,
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub payment_mint: Box<Account<'info, Mint>>,

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
    let total_supply = ctx.accounts.xyber_core.total_supply;

    let token_seed_vec = params.token_seed.key().to_bytes().to_vec();
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

    let raw_total_supply = total_supply * 10u64.pow(xyber_params::DECIMALS as u32);
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
    xyber_token.total_chains = params.total_chains;
    xyber_token.agent_wallet_pubkey = params.token_seed;

    Ok(())
}
