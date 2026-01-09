use anchor_lang::prelude::*;

use crate::error::*;
use crate::state::*;

pub fn set_admin(ctx: Context<AdminSet>, new_admin: Pubkey) -> Result<()> {
    let admin_settings_account = &mut ctx.accounts.admin_settings;
    admin_settings_account.admin = new_admin;
    Ok(())
}

#[derive(Accounts)]
pub struct AdminSet<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin @ CustomError::Unauthorized,
    )]
    pub admin_settings: Account<'info, AdminSettings>,
}
