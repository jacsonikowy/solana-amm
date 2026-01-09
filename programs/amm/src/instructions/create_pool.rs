use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{self, Mint, Token2022, TokenAccount},
};

use crate::state::*;

pub fn create_pool(ctx: Context<PoolCreation>) -> Result<()> {
    let pool_settings = &mut ctx.accounts.pool;

    pool_settings.token0 = ctx.accounts.token0.key();
    pool_settings.token1 = ctx.accounts.token1.key();
    pool_settings.liquidity = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct PoolCreation<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub token0: InterfaceAccount<'info, Mint>,
    pub token1: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [b"pool", token0.key().as_ref(), token1.key().as_ref()],
        space = 8 + Pool::INIT_SPACE,
        bump,
        constraint = token0.key() < token1.key()
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: READ ONLY
    #[account(
        seeds = [
            b"pool_authority",
            pool.key().as_ref(),
            token0.key().as_ref(),
            token1.key().as_ref(),
        ],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = token0,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program,
    )]
    pub token0_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = token1,
        associated_token::authority = pool_authority,
        associated_token::token_program = token_program,
    )]
    pub token1_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        has_one = admin,
        seeds = [b"admin"],
        bump
    )]
    pub admin_settings: Account<'info, AdminSettings>,

    #[account(
        init,
        payer = admin,
        seeds = [b"tokenliq", pool.key().as_ref(), token0.key().as_ref(), token1.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = pool_authority,
    )]
    pub token_liq: InterfaceAccount<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
