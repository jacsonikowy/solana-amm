use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::Token,
        token_interface::{self, Mint, TokenAccount, TransferChecked},
    },
    fixed::types::I64F64,
};

use crate::state::*;

pub fn swapExactInput(ctx: Context<SwapExactInput>, amount: u64) -> Result<()> {
    let mut amount_to_transfer_to_user = 0;

    if ctx.accounts.token_in.key() == ctx.accounts.token0.key() {
        let liquidity_token0 = ctx.accounts.token0_vault.amount;
        let liquidity_token1 = ctx.accounts.token1_vault.amount;
        // floor or ceil? i think rather ceil

        amount_to_transfer_to_user = I64F64::from_num(amount)
            .checked_mul(I64F64::from_num(liquidity_token1))
            .unwrap()
            .checked_div(
                I64F64::from_num(liquidity_token0)
                    .checked_add(I64F64::from_num(amount))
                    .unwrap(),
            )
            .unwrap()
            .to_num::<u64>();

        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    mint: ctx.accounts.token0.to_account_info(),
                    from: ctx.accounts.depositor_account_0.to_account_info(),
                    to: ctx.accounts.token0_vault.to_account_info(),
                    authority: ctx.accounts.swapper.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.token0.decimals,
        )?;

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
            amount_to_transfer_to_user,
            ctx.accounts.token1.decimals,
        )?;

        return Ok(());
    }

    if ctx.accounts.token_in.key() == ctx.accounts.token1.key() {
        let liquidity_token0 = ctx.accounts.token0_vault.amount;
        let liquidity_token1 = ctx.accounts.token1_vault.amount;

        amount_to_transfer_to_user = I64F64::from_num(amount)
            .checked_mul(I64F64::from_num(liquidity_token0))
            .unwrap()
            .checked_div(
                I64F64::from_num(liquidity_token1)
                    .checked_add(I64F64::from_num(amount))
                    .unwrap(),
            )
            .unwrap()
            .to_num::<u64>();

        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    mint: ctx.accounts.token1.to_account_info(),
                    from: ctx.accounts.depositor_account_1.to_account_info(),
                    to: ctx.accounts.token1_vault.to_account_info(),
                    authority: ctx.accounts.swapper.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.token1.decimals,
        )?;

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
            amount_to_transfer_to_user,
            ctx.accounts.token0.decimals,
        )?;

        return Ok(());
    }

    Ok(())
}

#[derive(Accounts)]
pub struct SwapExactInput<'info> {
    #[account(mut)]
    pub swapper: Signer<'info>,

    pub token0: InterfaceAccount<'info, Mint>,
    pub token1: InterfaceAccount<'info, Mint>,

    #[account(
        constraint = token_in.key() == token0.key() || token_in.key() == token1.key()
    )]
    pub token_in: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"pool", token0.key().as_ref(), token1.key().as_ref()],
        bump,
        has_one = token0,
        has_one = token1
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: readonly
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref(), token0.key().as_ref(), token1.key().as_ref()],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

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
        associated_token::authority = swapper
    )]
    pub depositor_account_0: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token1,
        associated_token::authority = swapper
    )]
    pub depositor_account_1: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
