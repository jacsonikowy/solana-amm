use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Decimals of tokens are not equal")]
    DecimalsNotEqual,
    #[msg("Invalid Liquidity")]
    InvalidLiquidity,
    #[msg("Math Overflow")]
    MathOverflow,
    #[msg("Deposit Zero amount token")]
    ZeroAmount,
}
