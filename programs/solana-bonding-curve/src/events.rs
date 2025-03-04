use anchor_lang::prelude::*;

#[event]
pub struct GraduationTriggered {
    pub buyer: Pubkey,
    pub escrow_balance: u64,
    pub vault: Pubkey,
    pub creator: Pubkey,
}
