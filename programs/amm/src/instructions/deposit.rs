use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{self, Mint, MintTo, Token2022, TokenAccount, TransferChecked},
};
use fixed::types::I64F64;

use crate::error::*;
use crate::state::*;

// dodaj liq
// wymintuj tokeny odpowiednie
// pobierz od usera dwa tokeny

pub fn deposit(ctx: Context<DepositLiquidity>, mut amount_a: u64, mut amount_b: u64) -> Result<()> {
    // token0 transfer

    let decimals0 = ctx.accounts.token0.decimals;
    let decimals1 = ctx.accounts.token1.decimals;

    if decimals0 != decimals1 {
        return err!(CustomError::DecimalsNotEqual);
    }

    if amount_a == 0 || amount_b == 0 {
        return err!(CustomError::ZeroAmount);
    }

    let vault_a = &ctx.accounts.token0_vault;
    let vault_b = &ctx.accounts.token1_vault;

    let pool_creation = vault_a.amount == 0 && vault_b.amount == 0;

    (amount_a, amount_b) = if pool_creation {
        (amount_a, amount_b)
    } else {
        let ratio = I64F64::from_num(vault_a.amount)
            .checked_mul(I64F64::from_num(vault_b.amount))
            .unwrap();
        if vault_a.amount > vault_b.amount {
            (
                I64F64::from_num(amount_b)
                    .checked_mul(ratio)
                    .unwrap()
                    .to_num::<u64>(),
                amount_b,
            )
        } else {
            (
                amount_a,
                I64F64::from_num(amount_a)
                    .checked_div(ratio)
                    .unwrap()
                    .to_num::<u64>(),
            )
        }
    };

    // write tests for insufficient funds
    // transfering token0
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.token0.to_account_info(),
                from: ctx.accounts.depositor_account_0.to_account_info(),
                to: ctx.accounts.token0_vault.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        amount_a,
        decimals0,
    )?;

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.token1.to_account_info(),
                from: ctx.accounts.depositor_account_1.to_account_info(),
                to: ctx.accounts.token1_vault.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        amount_b,
        decimals1,
    )?;

    // calc liquidity
    // sqrt(a*b)

    let liquidity = I64F64::from_num(amount_a)
        .checked_mul(I64F64::from_num(amount_b))
        .ok_or(CustomError::MathOverflow)?
        .sqrt()
        .checked_to_num::<u64>()
        .ok_or(CustomError::InvalidLiquidity)?;

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
    // transfer
    token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.tokenliq.to_account_info(),
                to: ctx.accounts.depositor_account_liq.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer_seeds,
        ),
        liquidity,
    )?;

    let mut pool = &mut ctx.accounts.pool;
    pool.liquidity += liquidity;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub token0: InterfaceAccount<'info, Mint>,
    pub token1: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [b"tokenliq", pool.key().as_ref(), token0.key().as_ref(), token1.key().as_ref()],
        bump,
    )]
    pub tokenliq: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token0,
        associated_token::authority = pool_authority,
    )]
    pub token0_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token1,
        associated_token::authority = pool_authority,
    )]
    pub token1_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token0,
        associated_token::authority = signer,
    )]
    pub depositor_account_0: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token1,
        associated_token::authority = signer,
    )]
    pub depositor_account_1: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = tokenliq,
        associated_token::authority = signer,
    )]
    pub depositor_account_liq: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"pool", token0.key().as_ref(), token1.key().as_ref()],
        bump,
        has_one = token0,
        has_one = token1
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: readolny
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref(), token0.key().as_ref(), token1.key().as_ref()],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
