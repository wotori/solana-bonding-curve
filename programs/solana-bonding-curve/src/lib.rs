use anchor_lang::prelude::*;
use anchor_lang::system_program::{self};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::mpl_token_metadata;
use anchor_spl::metadata::mpl_token_metadata::{
    instructions::{CreateV1, CreateV1InstructionArgs},
    types::TokenStandard,
};
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};

declare_id!("BqrcBeCGtK1qasqykoFydSEV31iEYTuPnUW1mpsptv5W");

#[program]
pub mod bonding_curve {
    use super::*;
    use anchor_lang::solana_program::program::invoke_signed;
    use anchor_lang::system_program::Transfer;

    // ------------------------------------------------------------------------
    // 1) CREATE TOKEN (no escrow here!)
    // ------------------------------------------------------------------------
    pub fn create_token_instruction(
        ctx: Context<CreateToken>,
        total_supply: u64,
        initial_mint_amount: u64,
        price_lamports: u64,
    ) -> Result<()> {
        // Store token data in OwnedToken
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.supply = total_supply;
        owned_token.price_lamports = price_lamports;
        // We do *not* set escrow_pda or escrow_bump here anymore!

        // Mint initial tokens to creator
        let bump = ctx.bumps.owned_token;
        let creator_key = ctx.accounts.creator.key();
        let token_seed_key = ctx.accounts.token_seed.key();
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

    // ------------------------------------------------------------------------
    // 2) INIT ESCROW (separate instruction)
    // ------------------------------------------------------------------------
    pub fn init_escrow_instruction(ctx: Context<InitEscrow>) -> Result<()> {
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.escrow_pda = ctx.accounts.escrow_pda.key();
        owned_token.escrow_bump = ctx.bumps.escrow_pda;
        Ok(())
    }

    // ------------------------------------------------------------------------
    // SET METADATA
    // ------------------------------------------------------------------------
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

        let bump = ctx.bumps.owned_token;
        let creator_key = ctx.accounts.creator.key();
        let token_seed_key = ctx.accounts.token_seed.key();
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

    // ------------------------------------------------------------------------
    // BUY TOKENS
    // ------------------------------------------------------------------------
    pub fn buy_instruction(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
        // 1) Check supply
        let owned_token = &ctx.accounts.owned_token;
        require!(
            amount <= owned_token.supply,
            CustomError::InsufficientTokenSupply
        );

        // 2) Calculate cost
        let decimals_base = 10_u64.pow(9);
        let total_cost = amount
            .checked_mul(owned_token.price_lamports)
            .ok_or(CustomError::MathOverflow)?
            .checked_div(decimals_base)
            .ok_or(CustomError::MathOverflow)?;

        // 3) Transfer lamports buyer -> escrow_pda
        {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.escrow_pda.to_account_info(),
                },
            );
            system_program::transfer(cpi_ctx, total_cost)?;
        }

        // 4) Mint `amount` tokens to buyer
        {
            let bump = ctx.bumps.owned_token;
            let creator_key = ctx.accounts.creator.key();
            let token_seed_key = ctx.accounts.token_seed.key();
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
                        to: ctx.accounts.buyer_token_account.to_account_info(),
                        authority: ctx.accounts.owned_token.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                amount,
            )?;
        }

        // 5) Decrease supply
        {
            let owned_token = &mut ctx.accounts.owned_token;
            owned_token.supply = owned_token
                .supply
                .checked_sub(amount)
                .ok_or(CustomError::MathOverflow)?;
        }

        Ok(())
    }

    // ------------------------------------------------------------------------
    // SELL TOKENS
    // ------------------------------------------------------------------------
    pub fn sell_instruction(ctx: Context<SellToken>, amount: u64) -> Result<()> {
        // 1) Burn user's tokens
        {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            );
            token::burn(cpi_ctx, amount)?;
        }

        // 2) Calculate lamports to return
        let owned_token = &ctx.accounts.owned_token;
        let decimals_base = 10_u64.pow(9);
        let total_return = amount
            .checked_mul(owned_token.price_lamports)
            .ok_or(CustomError::MathOverflow)?
            .checked_div(decimals_base)
            .ok_or(CustomError::MathOverflow)?;

        // 3) Transfer lamports from escrow_pda -> user
        {
            let bump = owned_token.escrow_bump;
            let creator_key = ctx.accounts.creator.key();
            let token_seed_key = ctx.accounts.token_seed.key();

            let escrow_seeds = &[
                b"escrow".as_ref(),
                creator_key.as_ref(),
                token_seed_key.as_ref(),
                &[bump],
            ];

            let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.escrow_pda.key(),
                &ctx.accounts.user.key(),
                total_return,
            );
            invoke_signed(
                &transfer_ix,
                &[
                    ctx.accounts.escrow_pda.to_account_info(),
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[escrow_seeds],
            )?;
        }

        // 4) Increase supply
        {
            let owned_token = &mut ctx.accounts.owned_token;
            owned_token.supply = owned_token
                .supply
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?;
        }

        Ok(())
    }
}

// ------------------------------------------------------------------------
// ACCOUNTS
// ------------------------------------------------------------------------

/// Our main data struct (PDA) storing supply & price.
#[account]
pub struct OwnedToken {
    pub supply: u64,
    pub price_lamports: u64,
    pub escrow_pda: Pubkey,
    pub escrow_bump: u8,
}
// size = 8 discriminator + 8 + 8 + 32 + 1 = 57 bytes
impl OwnedToken {
    pub const LEN: usize = 8 + 8 + 8 + 32 + 1;
}

// ------------------------------------------------------------------------
//  CreateToken
// ------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(total_supply: u64, initial_mint_amount: u64, price_lamports: u64)]
pub struct CreateToken<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    // OwnedToken
    #[account(
        init,
        payer = creator,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        space = OwnedToken::LEN
    )]
    pub owned_token: Account<'info, OwnedToken>,

    // Mint
    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = owned_token
    )]
    pub mint: Account<'info, Mint>,

    // Creator's ATA
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    // Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    // System
    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

// ------------------------------------------------------------------------
//  InitEscrow: separate instruction now
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct InitEscrow<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    // Must be the same OwnedToken that was just created
    #[account(
        mut,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(
        init,
        payer = creator,
        seeds = [b"escrow", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        owner = system_program::ID,
        space = 0
    )]
    /// CHECK: Escrow account
    pub escrow_pda: UncheckedAccount<'info>,

    // System
    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

// ------------------------------------------------------------------------
//  SetMetadata
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct SetMetadata<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub owned_token: Account<'info, OwnedToken>,

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

// ------------------------------------------------------------------------
//  BuyToken
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct BuyToken<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: seed
    #[account()]
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(
        mut,
        seeds = [b"escrow", creator.key().as_ref(), token_seed.key().as_ref()],
        bump = owned_token.escrow_bump,
        owner = system_program::ID
    )]
    /// CHECK: Escrow for SOL
    pub escrow_pda: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    // Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

// ------------------------------------------------------------------------
//  SellToken
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct SellToken<'info> {
    /// CHECK: seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: seed
    #[account()]
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(
        mut,
        seeds = [b"escrow", creator.key().as_ref(), token_seed.key().as_ref()],
        bump = owned_token.escrow_bump,
        owner = system_program::ID
    )]
    /// CHECK: escrow for SOL
    pub escrow_pda: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        has_one = mint
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

// ------------------------------------------------------------------------
//  Errors
// ------------------------------------------------------------------------
#[error_code]
pub enum CustomError {
    #[msg("Token supply is not enough to fulfill buy request.")]
    InsufficientTokenSupply,
    #[msg("Math overflow or underflow.")]
    MathOverflow,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Missing bump seed in bumps map.")]
    MissingBump,
}
