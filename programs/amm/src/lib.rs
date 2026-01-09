use anchor_lang::prelude::*;

declare_id!("3PX9c1PewnSzw69TPCb345uDhGR5FXtdDAAQnrBj7nh4");

mod error;
mod instructions;
mod state;

pub use instructions::*;

#[program]
pub mod amm {
    use super::*;

    pub fn init_admin(ctx: Context<InitAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::init_admin(ctx, new_admin)
    }

    pub fn set_admin(ctx: Context<AdminSet>, new_admin: Pubkey) -> Result<()> {
        instructions::set_admin(ctx, new_admin)
    }

    pub fn create_pool(ctx: Context<PoolCreation>) -> Result<()> {
        instructions::create_pool(ctx)
    }

    pub fn deposit(ctx: Context<DepositLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        instructions::deposit(ctx, amount_a, amount_b)
    }

    pub fn withdraw(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
        instructions::withdraw_liquidity(ctx, amount)
    }

    pub fn swapExactInput(ctx: Context<SwapExactInput>, amount: u64) -> Result<()> {
        instructions::swapExactInput(ctx, amount)
    }
}
