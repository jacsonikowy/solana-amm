use {
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
    solana_system_interface::program::ID as system_program_id,
    spl_associated_token_account::get_associated_token_address,
    spl_associated_token_account::get_associated_token_address_with_program_id,
};

use crate::utils::get_utils;

pub fn build_createPool_instruction(
    program_id: &Pubkey,
    admin: &Pubkey,
    token0: &Pubkey,
    token1: &Pubkey,
) -> Instruction {
    let discriminator = get_utils::get_discriminator("create_pool");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);

    let (admin_settings, _bump) = get_utils::get_admin_settings_pda(program_id);
    let (token0_vault_pda, _bump_token0_vault) =
        get_utils::get_token_vault_pda(program_id, admin, token0);
    let (token1_vault_pda, _bump_token1_vault) =
        get_utils::get_token_vault_pda(program_id, admin, token1);
    let (tokenliq_pda, _bump_tokenliq) =
        get_utils::get_tokenliq_pda(program_id, admin, token0, token1);
    let (pool_pda, _bump_pool) = get_utils::get_pool_pda(program_id, admin, token0, token1);
    let (pool_authority_pda, _bump_pool_authority) =
        get_utils::get_pool_authority_pda(program_id, &pool_pda, token0, token1);

    let token0_vault =
        get_associated_token_address_with_program_id(&pool_authority_pda, token0, &spl_token::id());
    let token1_vault =
        get_associated_token_address_with_program_id(&pool_authority_pda, token1, &spl_token::id());

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new_readonly(*token0, false),
            AccountMeta::new_readonly(*token1, false),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(pool_authority_pda, false),
            AccountMeta::new(token0_vault, false),
            AccountMeta::new(token1_vault, false),
            AccountMeta::new(admin_settings, false),
            AccountMeta::new(tokenliq_pda, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: instruction_data,
    }
}

pub fn build_deposit_instruction(
    program_id: &Pubkey,
    admin_pub: &Pubkey,
    depositor: &Pubkey,
    token0: &Pubkey,
    token1: &Pubkey,
    amount_a: &u64,
    amount_b: &u64,
) -> Instruction {
    let discriminator = get_utils::get_discriminator("deposit");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);

    let amount_a_bytes = amount_a.to_le_bytes();
    instruction_data.extend_from_slice(&amount_a_bytes);

    let amount_b_bytes = amount_b.to_le_bytes();
    instruction_data.extend_from_slice(&amount_b_bytes);

    let (tokenliq_pda, tokenliq_bump) =
        get_utils::get_tokenliq_pda(program_id, admin_pub, token0, token1);
    //token0 vault
    let (pool_pda, pool_bump) = get_utils::get_pool_pda(program_id, admin_pub, token0, token1);
    let (pool_authority_pda, pool_bump) =
        get_utils::get_pool_authority_pda(program_id, &pool_pda, token0, token1);
    let token0_vault = get_associated_token_address(&pool_authority_pda, token0);
    let token1_vault = get_associated_token_address(&pool_authority_pda, token1);

    let token0_depositor_ata = get_associated_token_address(depositor, token0);
    let token1_depositor_ata = get_associated_token_address(depositor, token1);

    let tokenliq_depositor_ata = get_associated_token_address(depositor, &tokenliq_pda);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*depositor, true),
            AccountMeta::new_readonly(*token0, false),
            AccountMeta::new_readonly(*token1, false),
            AccountMeta::new(tokenliq_pda, false),
            AccountMeta::new(token0_vault, false),
            AccountMeta::new(token1_vault, false),
            AccountMeta::new(token0_depositor_ata, false),
            AccountMeta::new(token1_depositor_ata, false),
            AccountMeta::new(tokenliq_depositor_ata, false),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(pool_authority_pda, false),
            // idk if its safe
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program_id, false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: instruction_data,
    }
}

pub fn build_withdraw_instruction(
    program_id: &Pubkey,
    admin_pub: &Pubkey,
    depositor_pub: &Pubkey,
    token0: &Pubkey,
    token1: &Pubkey,
    amount: &u64,
) -> Instruction {
    let discriminator = get_utils::get_discriminator("withdraw");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);

    let amount_bytes = amount.to_le_bytes();
    instruction_data.extend_from_slice(&amount_bytes);

    let (tokenliq_pda, tokenliq_bump) =
        get_utils::get_tokenliq_pda(program_id, admin_pub, token0, token1);
    //token0 vault
    let (pool_pda, pool_bump) = get_utils::get_pool_pda(program_id, admin_pub, token0, token1);
    let (pool_authority_pda, pool_bump) =
        get_utils::get_pool_authority_pda(program_id, &pool_pda, token0, token1);
    let token0_vault = get_associated_token_address(&pool_authority_pda, token0);
    let token1_vault = get_associated_token_address(&pool_authority_pda, token1);

    let token0_depositor_ata = get_associated_token_address(depositor_pub, token0);
    let token1_depositor_ata = get_associated_token_address(depositor_pub, token1);

    let tokenliq_depositor_ata = get_associated_token_address(depositor_pub, &tokenliq_pda);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*depositor_pub, true),
            AccountMeta::new_readonly(*token0, false),
            AccountMeta::new_readonly(*token1, false),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(pool_authority_pda, false),
            AccountMeta::new(tokenliq_pda, false),
            AccountMeta::new(token0_vault, false),
            AccountMeta::new(token1_vault, false),
            AccountMeta::new(token0_depositor_ata, false),
            AccountMeta::new(token1_depositor_ata, false),
            AccountMeta::new(tokenliq_depositor_ata, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: instruction_data,
    }
}

pub fn build_initAdmin_instruction(program_id: &Pubkey, admin: &Pubkey) -> Instruction {
    let discriminator = get_utils::get_discriminator("init_admin");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);

    let admin_pubkey = admin.to_bytes();
    instruction_data.extend_from_slice(&admin_pubkey);

    let (admin_settings, _bump) = get_utils::get_admin_settings_pda(program_id);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(admin_settings, false),
            AccountMeta::new_readonly(system_program_id, false),
        ],
        data: instruction_data,
    }
}

pub fn build_setAdmin_instruction(
    program_id: &Pubkey,
    admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let discriminator = get_utils::get_discriminator("set_admin");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);

    let new_admin_pubkey = new_admin.to_bytes();
    instruction_data.extend_from_slice(&new_admin_pubkey);

    let (admin_settings, _bump) = get_utils::get_admin_settings_pda(program_id);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(admin_settings, false),
        ],
        data: instruction_data,
    }
}

pub fn build_swapExactInput_instruction(
    program_id: &Pubkey,
    admin_pub: &Pubkey,
    swapper: &Pubkey,
    token0: &Pubkey,
    token1: &Pubkey,
    token_in: &Pubkey,
    amount: &u64,
) -> Instruction {
    let discriminator = get_utils::get_discriminator("swapExactInput");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);

    let amount_bytes = amount.to_le_bytes();
    instruction_data.extend_from_slice(&amount_bytes);

    let (pool_pda, pool_bump) = get_utils::get_pool_pda(program_id, admin_pub, token0, token1);
    let (pool_authority_pda, pool_bump) =
        get_utils::get_pool_authority_pda(program_id, &pool_pda, token0, token1);
    let token0_vault = get_associated_token_address(&pool_authority_pda, token0);
    let token1_vault = get_associated_token_address(&pool_authority_pda, token1);

    let token0_swapper_ata = get_associated_token_address(swapper, token0);
    let token1_swapper_ata = get_associated_token_address(swapper, token1);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*swapper, true),
            AccountMeta::new_readonly(*token0, false),
            AccountMeta::new_readonly(*token1, false),
            AccountMeta::new_readonly(*token_in, false),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(pool_authority_pda, false),
            AccountMeta::new(token0_vault, false),
            AccountMeta::new(token1_vault, false),
            AccountMeta::new(token0_swapper_ata, false),
            AccountMeta::new(token1_swapper_ata, false),
            AccountMeta::new(spl_token::id(), false),
        ],
        data: instruction_data,
    }
}
