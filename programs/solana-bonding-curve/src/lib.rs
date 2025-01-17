use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::mpl_token_metadata;
use anchor_spl::metadata::mpl_token_metadata::{
    instructions::{CreateV1, CreateV1InstructionArgs},
    types::TokenStandard,
};
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};

declare_id!("Da4dAJgYgs6Z4pcWWZzzvpdprtUB9hUDvoHkyJpQNYBz");

#[program]
pub mod bonding_curve {
    use super::*;
    use anchor_lang::solana_program::program::invoke_signed;
    use anchor_lang::system_program::Transfer;

    // ------------------------------------------------------------------------
    // CREATE TOKEN
    // ------------------------------------------------------------------------
    pub fn create_token_instruction(
        ctx: Context<CreateToken>,
        total_supply: u64,
        initial_mint_amount: u64,
        price_lamports: u64,
    ) -> Result<()> {
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.supply = total_supply;
        owned_token.price_lamports = price_lamports;

        let creator_key = ctx.accounts.creator.key();
        let token_seed_key = ctx.accounts.token_seed.key();
        let bump = ctx.bumps.owned_token;

        let signer_seeds = &[
            b"owned_token".as_ref(),
            creator_key.as_ref(),
            token_seed_key.as_ref(),
            &[bump],
        ];

        // Mint initial supply to creator
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

    // ------------------------------------------------------------------------
    // BUY TOKENS
    // ------------------------------------------------------------------------
    pub fn buy_instruction(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
        //
        // STEP 1: Do checks/maths with an immutable reference
        //
        // We cannot hold a mutable reference to `owned_token` if we also need
        // to pass it immutably to the CPI context. So do our checks using
        // an immutable reference first:
        //
        {
            let owned_token = &ctx.accounts.owned_token; // IMMUTABLE borrow
            require!(
                amount <= owned_token.supply,
                CustomError::InsufficientTokenSupply
            );
        }

        // Calculate cost in lamports
        let total_cost = {
            let owned_token = &ctx.accounts.owned_token; // IMMUTABLE again
            amount
                .checked_mul(owned_token.price_lamports)
                .ok_or(CustomError::MathOverflow)?
        };

        //
        // STEP 2: CPI calls referencing `owned_token.to_account_info()`
        //
        {
            // 2a) Transfer lamports from buyer to the OwnedToken PDA
            let cpi_ctx = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.owned_token.to_account_info(),
                },
            );
            anchor_lang::system_program::transfer(cpi_ctx, total_cost)?;
        }
        {
            // 2b) Mint tokens from the OwnedToken PDA to buyer
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
                        to: ctx.accounts.buyer_token_account.to_account_info(),
                        authority: ctx.accounts.owned_token.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                amount,
            )?;
        }

        //
        // STEP 3: Finally, re-borrow owned_token as mutable to update supply
        //
        {
            let owned_token = &mut ctx.accounts.owned_token; // MUTABLE borrow
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
        //
        // STEP 1: Burn the user's tokens (no need to mutate `OwnedToken` yet)
        //
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

        //
        // STEP 2: Transfer lamports from OwnedToken PDA to user
        //
        // We'll do the math with an immutable borrow, then do the CPI.
        //
        let total_return = {
            let owned_token = &ctx.accounts.owned_token; // IMMUTABLE borrow
            amount
                .checked_mul(owned_token.price_lamports)
                .ok_or(CustomError::MathOverflow)?
        };
        {
            let creator_key = ctx.accounts.creator.key();
            let token_seed_key = ctx.accounts.token_seed.key();
            let bump = ctx.bumps.owned_token;
            let signer_seeds = &[
                b"owned_token".as_ref(),
                creator_key.as_ref(),
                token_seed_key.as_ref(),
                &[bump],
            ];

            let pda_account_info = ctx.accounts.owned_token.to_account_info();
            let user_account_info = ctx.accounts.user.to_account_info();
            let system_program_info = ctx.accounts.system_program.to_account_info();

            let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                &pda_account_info.key(),
                &user_account_info.key(),
                total_return,
            );
            invoke_signed(
                &transfer_ix,
                &[
                    pda_account_info.clone(),
                    user_account_info.clone(),
                    system_program_info.clone(),
                ],
                &[signer_seeds],
            )?;
        }

        //
        // STEP 3: Now we can mutate `owned_token.supply`
        //
        {
            let owned_token = &mut ctx.accounts.owned_token; // MUTABLE borrow
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

/// Stores total supply + fixed price in lamports
#[account]
pub struct OwnedToken {
    pub supply: u64,
    pub price_lamports: u64,
}

impl OwnedToken {
    // Discriminator (8) + supply (8) + price (8) = 24
    pub const LEN: usize = 8 + 8 + 8;
}

// CreateToken Context
#[derive(Accounts)]
#[instruction(total_supply: u64, initial_mint_amount: u64, price_lamports: u64)]
pub struct CreateToken<'info> {
    /// CHECK: seed
    #[account()]
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

// SetMetadata Context
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
    /// CHECK: Metaplex Token Metadata program
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,

    #[account(address = anchor_spl::token::ID)]
    /// CHECK: SPL Token
    pub token_program: UncheckedAccount<'info>,

    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar Instructions
    pub sysvar_instructions: UncheckedAccount<'info>,
}

// BuyToken Context
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

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

// SellToken Context
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

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        has_one = mint
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

// ------------------------------------------------------------------------
// CUSTOM ERRORS
// ------------------------------------------------------------------------
#[error_code]
pub enum CustomError {
    #[msg("Token supply is not enough to fulfill buy request.")]
    InsufficientTokenSupply,
    #[msg("Math overflow or underflow.")]
    MathOverflow,
}
