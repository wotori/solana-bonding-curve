use anchor_lang::prelude::*;

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
