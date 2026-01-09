use {
    fixed::types::I64F64,
    sha2::{Digest, Sha256},
    solana_sdk::pubkey::Pubkey,
};

pub fn get_expected_liquidity(amount_a: &u64, amount_b: &u64) -> Result<u64, String> {
    let a = I64F64::from_num(*amount_a);
    let b = I64F64::from_num(*amount_b);

    let product = a.checked_mul(b).ok_or_else(|| "Overflow".to_string())?;

    let sqrt_res = product.sqrt();

    let liq = sqrt_res
        .checked_to_num::<u64>()
        .ok_or_else(|| "Result doesn't fit in u64".to_string())?;

    Ok(liq)
}

pub fn get_discriminator(instruction_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", instruction_name));
    let result = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&result[..8]);
    discriminator
}

pub fn get_admin_settings_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"admin"], program_id)
}

pub fn get_tokenliq_pda(
    program_id: &Pubkey,
    admin_pub: &Pubkey,
    token0: &Pubkey,
    token1: &Pubkey,
) -> (Pubkey, u8) {
    let (pool_pda, _bump) = get_pool_pda(&program_id, &admin_pub, &token0, &token1);

    Pubkey::find_program_address(
        &[
            b"tokenliq",
            pool_pda.as_ref(),
            token0.as_ref(),
            token1.as_ref(),
        ],
        program_id,
    )
}

pub fn get_token_vault_pda(
    program_id: &Pubkey,
    admin_pub: &Pubkey,
    token: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[admin_pub.as_ref(), token.as_ref()], program_id)
}

pub fn get_pool_pda(
    program_id: &Pubkey,
    admin_pub: &Pubkey,
    token0: &Pubkey,
    token1: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool", token0.as_ref(), token1.as_ref()], program_id)
}

pub fn get_pool_authority_pda(
    program_id: &Pubkey,
    pool: &Pubkey,
    token0: &Pubkey,
    token1: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"pool_authority",
            pool.as_ref(),
            token0.as_ref(),
            token1.as_ref(),
        ],
        program_id,
    )
}
