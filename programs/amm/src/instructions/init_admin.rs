use anchor_lang::prelude::*;

use crate::error::*;
use crate::state::*;

pub fn init_admin(ctx: Context<InitAdmin>, new_admin: Pubkey) -> Result<()> {
    let admin_settings_account = &mut ctx.accounts.admin_settings;
    admin_settings_account.admin = new_admin;
    Ok(())
}

#[derive(Accounts)]
pub struct InitAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32,
        seeds = [b"admin"],
        bump
    )]
    pub admin_settings: Account<'info, AdminSettings>,

    pub system_program: Program<'info, System>,
}
