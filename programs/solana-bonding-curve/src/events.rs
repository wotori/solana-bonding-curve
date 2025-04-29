use anchor_lang::prelude::*;

#[event]
pub struct GraduationTriggered {
    pub buyer: Pubkey,
    pub escrow_balance: u64,
    pub vault: Pubkey,
    pub creator: Pubkey,
    pub escrow: Pubkey,
    pub token_seed: Pubkey,
}

#[event]
pub struct XyberSwapEvent {
    pub ix_type: XyberInstructionType,
    pub token_seed: Pubkey,
    pub user: Pubkey,
    pub base_amount: u64,
    pub token_amount: u64,
    pub vault_token_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum XyberInstructionType {
    BuyExactIn = 0,
    BuyExactOut = 1,
    SellExactIn = 2,
    SellExactOut = 3,
}
