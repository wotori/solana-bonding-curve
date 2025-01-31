use anchor_lang::{prelude::*, solana_program::system_program};

use crate::OwnedToken;

#[derive(Accounts)]
pub struct InitEscrow<'info> {
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

pub fn init_escrow_instruction(ctx: Context<InitEscrow>) -> Result<()> {
    let omni_token = &mut ctx.accounts.omni_token;
    omni_token.escrow_pda = ctx.accounts.escrow_pda.key();
    omni_token.escrow_bump = ctx.bumps.escrow_pda;
    Ok(())
}
