use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{self, BurnChecked, Mint, MintTo, Token2022, TokenAccount, TransferChecked},
};
use fixed::types::I64F64;

use crate::error::*;
use crate::state::*;

pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
    // calc how much user gets token_0 and token_1
    // burn tokenliq
    // transfer both tokens
    //
    let pool_key = ctx.accounts.pool.key();
    let token0_key = ctx.accounts.token0.key();
    let token1_key = ctx.accounts.token1.key();

    let authority_bump = ctx.bumps.pool_authority;
    let authority_seeds = &[
        "pool_authority".as_bytes(),
        pool_key.as_ref(),
        token0_key.as_ref(),
        token1_key.as_ref(),
        &[authority_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];

    let amount_a = I64F64::from_num(amount)
        .checked_mul(I64F64::from_num(ctx.accounts.token0_vault.amount))
        .unwrap()
        .checked_div(I64F64::from_num(ctx.accounts.tokenliq.supply))
        .unwrap()
        .floor()
        .to_num::<u64>();

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.token0.to_account_info(),
                from: ctx.accounts.token0_vault.to_account_info(),
                to: ctx.accounts.depositor_account_0.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_a,
        ctx.accounts.token1.decimals,
    )?;

    let amount_b = I64F64::from_num(amount)
        .checked_mul(I64F64::from_num(ctx.accounts.token1_vault.amount))
        .unwrap()
        .checked_div(I64F64::from_num(ctx.accounts.tokenliq.supply))
        .unwrap()
        .floor()
        .to_num::<u64>();

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.token1.to_account_info(),
                from: ctx.accounts.token1_vault.to_account_info(),
                to: ctx.accounts.depositor_account_1.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_b,
        ctx.accounts.token1.decimals,
    )?;

    token_interface::burn_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            BurnChecked {
                mint: ctx.accounts.tokenliq.to_account_info(),
                from: ctx.accounts.depositor_account_liq.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.tokenliq.decimals,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    pub token0: InterfaceAccount<'info, Mint>,
    pub token1: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [
            b"pool",
            token0.key().as_ref(),
            token1.key().as_ref()
        ],
        bump,
        has_one = token0,
        has_one = token1
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: read only
    #[account(
        seeds = [
            b"pool_authority",
            pool.key().as_ref(),
            token0.key().as_ref(),
            token1.key().as_ref()
        ],
        bump,
    )]
    pub pool_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"tokenliq", pool.key().as_ref(), token0.key().as_ref(), token1.key().as_ref()],
        bump,
    )]
    pub tokenliq: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token0,
        associated_token::authority = pool_authority
    )]
    pub token0_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token1,
        associated_token::authority = pool_authority
    )]
    pub token1_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token0,
        associated_token::authority = depositor
    )]
    pub depositor_account_0: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token1,
        associated_token::authority = depositor
    )]
    pub depositor_account_1: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = tokenliq,
        associated_token::authority = depositor
    )]
    pub depositor_account_liq: InterfaceAccount<'info, TokenAccount>,

    // TODO: check if user can delete his own ata
    pub token_program: Program<'info, Token>,
}
