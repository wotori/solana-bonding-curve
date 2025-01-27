use anchor_lang::prelude::*;

mod curves;
mod errors;
mod omni_params;

mod instructions;

use curves::SmoothBondingCurve;
use instructions::*;

declare_id!("GMjvbDmasN1FyYD6iGfj5u8EETdk9gTQnyoZUQA4PVGT");

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

#[program]
pub mod bonding_curve {
    use super::*;

    // (1.1) CREATE TOKEN
    pub fn create_token_instruction(ctx: Context<CreateToken>) -> Result<()> {
        instructions::create_token_instruction(ctx)
    }

    // (1.2) INIT ESCROW
    //   - Create the escrow PDA
    pub fn init_escrow_instruction(ctx: Context<InitEscrow>) -> Result<()> {
        instructions::init_escrow_instruction(ctx)
    }

    // (1.3) MINT INITIAL TOKENS
    //   - Transfer lamports from creator -> escrow
    //   - Use bonding curve to calculate how many tokens that buys
    //   - Mint them to creator's ATA
    //   - Subtract from OwnedToken.supply
    pub fn mint_initial_tokens_instruction(
        ctx: Context<MintInitialTokens>,
        deposit_lamports: u64,
    ) -> Result<()> {
        instructions::mint::mint_initial_tokens_instruction(ctx, deposit_lamports)
    }

    // (1.4) SET METADATA
    pub fn set_metadata_instruction(
        ctx: Context<SetMetadata>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::set_metadata_instruction(ctx, name, symbol, uri)
    }

    pub fn buy_instruction(ctx: Context<BuyToken>, lamports: u64) -> Result<()> {
        instructions::buy_instruction(ctx, lamports)
    }

    pub fn sell_instruction(ctx: Context<SellToken>, normalized_token_amount: u64) -> Result<()> {
        instructions::sell_instruction(ctx, normalized_token_amount)
    }
}
