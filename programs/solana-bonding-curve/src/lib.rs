use anchor_lang::prelude::*;

declare_id!("E48ijHDaZqdVBiGgCGGJRQTM373TsjYPGv8rpVzu4P9R");

#[program]
pub mod solana_bonding_curve {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
