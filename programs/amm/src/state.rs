use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct AdminSettings {
    pub admin: Pubkey,
}

#[account]
#[derive(Default)]
pub struct Pool {
    pub token0: Pubkey,
    pub token1: Pubkey,
    pub liquidity: u64,
}

impl Pool {
    pub const INIT_SPACE: usize = 72;
}
