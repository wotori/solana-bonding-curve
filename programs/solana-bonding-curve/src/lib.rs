use anchor_lang::prelude::*;
use anchor_lang::{solana_program::native_token::LAMPORTS_PER_SOL, system_program};

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::mpl_token_metadata;
use anchor_spl::metadata::mpl_token_metadata::{
    instructions::{CreateV1, CreateV1InstructionArgs},
    types::TokenStandard,
};
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};

pub mod curves;
pub mod errors;
pub mod omni_params;

use curves::traits::BondingCurveTrait;
use curves::SmoothBondingCurve;

declare_id!("GMjvbDmasN1FyYD6iGfj5u8EETdk9gTQnyoZUQA4PVGT");

#[program]
pub mod bonding_curve {
    use super::*;
    use anchor_lang::solana_program::program::invoke_signed;
    use anchor_lang::system_program::Transfer;
    use errors::CustomError;

    // ------------------------------------------------------------------------
    // (1) CREATE TOKEN
    //   - Create and initialize the OwnedToken PDA
    //   - Create the Mint (with authority = OwnedToken PDA)
    //   - Create the creator's Associated Token Account (ATA)
    //   - Initialize the bonding curve and supply in OwnedToken
    // ------------------------------------------------------------------------
    pub fn create_token_instruction(ctx: Context<CreateToken>) -> Result<()> {
        let owned_token = &mut ctx.accounts.owned_token;
        // Example: set initial supply to a large number so can "subtract" from it later.
        owned_token.supply = 1_073_000_191;

        // Initialize bonding curve
        owned_token.bonding_curve = SmoothBondingCurve {
            a: 1_073_000_191,
            k: 32_190_005_730 * LAMPORTS_PER_SOL as u128,
            c: 30 * LAMPORTS_PER_SOL,
            x: 0,
        };

        Ok(())
    }

    // ------------------------------------------------------------------------
    // (2) INIT ESCROW
    //   - Create the escrow PDA
    // ------------------------------------------------------------------------
    pub fn init_escrow_instruction(ctx: Context<InitEscrow>) -> Result<()> {
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.escrow_pda = ctx.accounts.escrow_pda.key();
        owned_token.escrow_bump = ctx.bumps.escrow_pda;
        Ok(())
    }

    // ------------------------------------------------------------------------
    // (3) MINT INITIAL TOKENS
    //   - Transfer lamports from creator -> escrow
    //   - Use bonding curve to calculate how many tokens that buys
    //   - Mint them to creator's ATA
    //   - Subtract from OwnedToken.supply
    // ------------------------------------------------------------------------
    pub fn mint_initial_tokens_instruction(
        ctx: Context<MintInitialTokens>,
        deposit_lamports: u64,
    ) -> Result<()> {
        // --------------------------------------------------------------------
        // 1) Transfer lamports from `creator` to Escrow PDA
        // --------------------------------------------------------------------
        msg!(
            "DEBUG: Starting mint_initial_tokens_instruction. deposit_lamports={}",
            deposit_lamports
        );
        msg!(
            "DEBUG: Escrow PDA balance BEFORE transfer: {} lamports",
            ctx.accounts.escrow_pda.to_account_info().lamports()
        );
        msg!(
            "DEBUG: Creator balance BEFORE transfer: {} lamports",
            ctx.accounts.creator.to_account_info().lamports()
        );

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.escrow_pda.to_account_info(),
            },
        );
        system_program::transfer(cpi_ctx, deposit_lamports)?;

        msg!(
            "DEBUG: Transfer SUCCESS. Escrow PDA balance AFTER transfer: {} lamports",
            ctx.accounts.escrow_pda.to_account_info().lamports()
        );
        msg!(
            "DEBUG: Creator balance AFTER transfer: {} lamports",
            ctx.accounts.creator.to_account_info().lamports()
        );

        // --------------------------------------------------------------------
        // 2) Calculate the token amount via the bonding curve
        // --------------------------------------------------------------------
        msg!("DEBUG: Calling buy_exact_input() in the bonding curve...");
        let minted_tokens_u128 = ctx
            .accounts
            .owned_token
            .bonding_curve
            .buy_exact_input(deposit_lamports);

        msg!(
            "DEBUG: buy_exact_input returned minted_tokens_u128={}",
            minted_tokens_u128
        );

        require!(
            minted_tokens_u128 <= u64::MAX as u128,
            CustomError::MathOverflow
        );

        let human_readable_tokens = minted_tokens_u128 as u64;
        msg!(
            "DEBUG: minted_tokens_u64={} (will pass this to token::mint_to)",
            human_readable_tokens
        );

        // --------------------------------------------------------------------
        // 3) Mint these tokens to the creator's ATA
        // --------------------------------------------------------------------
        let bump = ctx.bumps.owned_token;
        let creator_key = ctx.accounts.creator.key();
        let token_seed_key = ctx.accounts.token_seed.key();

        msg!("DEBUG: Bump = {}", bump);
        msg!("DEBUG: Creator Pubkey = {}", creator_key);
        msg!("DEBUG: Token Seed Pubkey = {}", token_seed_key);

        let signer_seeds = &[
            b"owned_token".as_ref(),
            creator_key.as_ref(),
            token_seed_key.as_ref(),
            &[bump],
        ];

        let minted_tokens_u64 = human_readable_tokens * 10_u64.pow(omni_params::DECIMALS as u32);
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
            minted_tokens_u64,
        )?;
        msg!("DEBUG: mint_to SUCCESS!");

        // --------------------------------------------------------------------
        // 4) Reduce supply
        // --------------------------------------------------------------------
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.supply = owned_token
            .supply
            .checked_sub(human_readable_tokens)
            .ok_or(CustomError::MathOverflow)?;
        msg!("DEBUG: owned_token.supply AFTER sub={}", owned_token.supply);

        msg!("DEBUG: Instruction complete. Returning Ok(()).");
        Ok(())
    }

    // ------------------------------------------------------------------------
    // (4) SET METADATA
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
            decimals: Some(omni_params::DECIMALS),
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
    pub fn buy_instruction(ctx: Context<BuyToken>, lamports: u64) -> Result<()> {
        // 1) Calculate cost based on the bonding curve + check supply
        let tokens_u128 = {
            let owned_token = &mut ctx.accounts.owned_token;
            let tokens_u128 = owned_token.bonding_curve.buy_exact_input(lamports);
            require!(
                tokens_u128 as u64 <= owned_token.supply,
                CustomError::InsufficientTokenSupply
            );
            tokens_u128
        };

        // 2) Transfer lamports from buyer -> escrow
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.escrow_pda.to_account_info(),
            },
        );
        system_program::transfer(cpi_ctx, lamports)?;

        // 3) Mint tokens to the buyer
        let bump = ctx.bumps.owned_token;
        let creator_key = ctx.accounts.creator.key();
        let token_seed_key = ctx.accounts.token_seed.key();
        let signer_seeds = &[
            b"owned_token".as_ref(),
            creator_key.as_ref(),
            token_seed_key.as_ref(),
            &[bump],
        ];

        require!(tokens_u128 <= u64::MAX as u128, CustomError::MathOverflow);
        let human_readable_tokens = tokens_u128 as u64;
        let minted_tokens_u64 = human_readable_tokens * 10_u64.pow(omni_params::DECIMALS as u32);

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
            minted_tokens_u64,
        )?;

        // 4) Update the supply
        let owned_token = &mut ctx.accounts.owned_token;
        owned_token.supply = owned_token
            .supply
            .checked_sub(human_readable_tokens as u64)
            .ok_or(CustomError::MathOverflow)?;

        Ok(())
    }

    // ------------------------------------------------------------------------
    // SELL TOKENS
    // ------------------------------------------------------------------------
    pub fn sell_instruction(ctx: Context<SellToken>, normalized_token_mount: u64) -> Result<()> {
        // 1) Burn user's tokens
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );

        let tokens_to_burn = normalized_token_mount
            .checked_mul(10_u64.pow(omni_params::DECIMALS as u32))
            .ok_or(CustomError::MathOverflow)?;

        token::burn(cpi_ctx, tokens_to_burn)?;

        // 2) Calculate how much SOL to return
        let owned_token = &mut ctx.accounts.owned_token;
        let lamports_return = owned_token
            .bonding_curve
            .sell_exact_input(normalized_token_mount as u128);

        // 3) Transfer SOL from escrow -> user
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
            lamports_return,
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

        // 4) Increase supply
        owned_token.supply = owned_token
            .supply
            .checked_add(normalized_token_mount)
            .ok_or(CustomError::MathOverflow)?;

        Ok(())
    }
}

// ------------------------------------------------------------------------
// ACCOUNTS
// ------------------------------------------------------------------------

/// The PDA storing bonding curve data, supply, and escrow info.
#[account]
pub struct OwnedToken {
    pub supply: u64,
    pub bonding_curve: SmoothBondingCurve,
    pub escrow_pda: Pubkey,
    pub escrow_bump: u8,
}
impl OwnedToken {
    // 8 discriminator + 8 supply + 40 bonding_curve + 32 escrow + 1 bump = 89
    pub const LEN: usize = 89;
}
// ------------------------------------------------------------------------
// (1) CreateToken (no escrow logic here)
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct CreateToken<'info> {
    /// CHECK: arbitrary seed
    #[account()]
    pub token_seed: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        seeds = [b"owned_token", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        space = OwnedToken::LEN
    )]
    pub owned_token: Account<'info, OwnedToken>,

    #[account(
        init,
        payer = creator,
        mint::decimals = omni_params::DECIMALS,
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

    // Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    // Not used in create_token, but typically
    // required by the same client code flow
    /// CHECK: Escrow account
    pub escrow_pda: UncheckedAccount<'info>,

    // System
    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

// ------------------------------------------------------------------------
// (2) InitEscrow
// ------------------------------------------------------------------------
#[derive(Accounts)]
pub struct InitEscrow<'info> {
    /// CHECK: arbitrary seed
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

    #[account(
        init,
        payer = creator,
        seeds = [b"escrow", creator.key().as_ref(), token_seed.key().as_ref()],
        bump,
        owner = system_program::ID,
        space = 0
    )]
    /// CHECK: escrow for SOL
    pub escrow_pda: UncheckedAccount<'info>,

    // System
    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

// ------------------------------------------------------------------------
// (3) MintInitialTokens
// ------------------------------------------------------------------------
#[derive(Accounts)]
#[instruction(deposit_lamports: u64)]
pub struct MintInitialTokens<'info> {
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

    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,

    // Programs
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::ID)]
    /// CHECK: System Program
    pub system_program: UncheckedAccount<'info>,
}

// ------------------------------------------------------------------------
// (4) SetMetadata
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
// BuyToken
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
// SellToken
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
