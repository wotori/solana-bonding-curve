use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

pub static DECIMALS: u8 = 9;

pub static TOTAL_TOKENS: u64 = 1_073_000_191;
pub static VIRTUAL_POOL_OFFSET: u64 = 30 * LAMPORTS_PER_SOL;
pub static BONDING_SCALE_FACTOR: u128 = 32_190_005_730 * (LAMPORTS_PER_SOL as u128);
