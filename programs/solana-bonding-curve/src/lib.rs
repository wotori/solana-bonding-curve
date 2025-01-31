use anchor_lang::prelude::*;

mod curves;
mod errors;
mod xyber_params;

mod instructions;

use crate::xyber_params::CreateTokenParams;
use curves::SmoothBondingCurve;
use instructions::*;

declare_id!("GMjvbDmasN1FyYD6iGfj5u8EETdk9gTQnyoZUQA4PVGT");

#[account]
pub struct XyberToken {
    pub supply: u64,
    // pub decimals: u8,
    pub grad_threshold: u16,

    pub bonding_curve: SmoothBondingCurve,

    pub escrow_pda: Pubkey,
    pub escrow_bump: u8,

    pub accepted_base_mint: Pubkey,
}

impl XyberToken {
    pub const LEN: usize = 
        8  // Discriminator (account type)
        + 8  // supply (u64)
        // + 1  // decimals (u8)
        + 2  // grad_threshold (u16)
        + 40 // bonding_curve (struct)
        + 32 // escrow_pda (Pubkey)
        + 1  // escrow_bump (u8)
        + 32 // accepted_base_mint (Pubkey)
    ;
}

#[program]
pub mod bonding_curve {

    use super::*;

    // (1.1) CREATE TOKEN
    pub fn create_token_instruction(
        ctx: Context<CreateToken>,
        params: CreateTokenParams,
    ) -> Result<()> {
        instructions::create_token_instruction(ctx, params)
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
    //   - Subtract from XyberToken.supply
    pub fn mint_initial_tokens_instruction(
        ctx: Context<MintInitialTokens>,
        deposit_lamports: u64,
    ) -> Result<()> {
        instructions::mint_initial_tokens_instruction(ctx, deposit_lamports)
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

    pub fn buy_exact_input_instruction(ctx: Context<BuyToken>, lamports: u64) -> Result<()> {
        instructions::buy_exact_input_instruction(ctx, lamports)
    }

    pub fn buy_exact_output_instruction(ctx: Context<BuyToken>, tokens_out: u64) -> Result<()> {
        instructions::buy_exact_output_instruction(ctx, tokens_out)
    }

    pub fn sell_exact_input_instruction(
        ctx: Context<SellToken>,
        normalized_token_amount: u64,
    ) -> Result<()> {
        instructions::sell_exact_input_instruction(ctx, normalized_token_amount)
    }

    pub fn sell_exact_output_instruction(ctx: Context<SellToken>, lamports: u64) -> Result<()> {
        instructions::sell_exact_output_instruction(ctx, lamports)
    }
}
