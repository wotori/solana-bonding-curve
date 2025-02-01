use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Custom error: Token supply is not enough to fulfill buy request")]
    InsufficientTokenSupply,

    #[msg("Provide a smaller amount. Use normalized tokens (e.g., raw value / 10 ** decimals).")]
    MathOverflow,

    #[msg("Unauthorized: Caller is not authorized to perform this action.")]
    Unauthorized,

    #[msg("Liquidity not graduated: pool has not reached the required threshold.")]
    LiquidityNotGraduated,
}
