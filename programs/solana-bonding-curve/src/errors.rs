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

    #[msg("Insufficient token balance in the vault to fulfill the request.")]
    InsufficientTokenVaultBalance,

    #[msg("Insufficient escrow balance: Not enough tokens in escrow to complete the operation.")]
    InsufficientEscrowBalance,

    #[msg("Token has graduated: The bonding curve is no longer active as the token is now listed on a DEX.")]
    TokenIsGraduated,

    #[msg("Invalid seed: the provided seed must be exactly 32 bytes in length.")]
    InvalidSeed,
}
