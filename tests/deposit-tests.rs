use anchor_spl::token::spl_token::solana_program::program_option::COption;
use anchor_spl::token_2022::ID as TOKEN_2022_ID;
use borsh::BorshDeserialize;
use litesvm::LiteSVM;
use litesvm_token::{
    spl_token::{native_mint::DECIMALS, state::Account},
    CreateAssociatedTokenAccount, CreateMint, MintTo,
};
use sha2::{Digest, Sha256};
use solana_program::program_pack::Pack;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use solana_system_interface::program::ID as system_program_id;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022_interface::generic_token_account::GenericTokenAccount;

use fixed::types::I64F64;

mod utils;

#[derive(Debug, BorshDeserialize)]
struct AdminSettings {
    pub admin: Pubkey,
}

#[derive(Debug, BorshDeserialize)]
struct Pool {
    pub token0: Pubkey,
    pub token1: Pubkey,
    pub liquidity: u64,
}

#[test]
fn test_successful_deposit() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let depositor = Keypair::new();

    let program_keypair = read_keypair_file("../../target/deploy/amm-keypair.json")
        .expect("Failed to read program keypair");
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../target/deploy/amm.so");
    svm.add_program(program_id, program_bytes);

    svm.airdrop(&admin.pubkey(), 1000_000_000).unwrap();
    svm.airdrop(&depositor.pubkey(), 1_000_000_000_000).unwrap();

    // First we set admin
    let tx_result = utils::handlers::handle_init_admin(&mut svm, &program_id, &admin);
    assert!(
        tx_result.is_ok(),
        "Create transaction failed: {:?}",
        tx_result.err()
    );

    let token0 = CreateMint::new(&mut svm, &admin)
        .authority(&admin.pubkey())
        .decimals(DECIMALS)
        .send()
        .unwrap();

    let mut token1 = CreateMint::new(&mut svm, &admin)
        .authority(&admin.pubkey())
        .decimals(DECIMALS)
        .send()
        .unwrap();

    while token1 <= token0 {
        token1 = CreateMint::new(&mut svm, &admin)
            .authority(&admin.pubkey())
            .decimals(DECIMALS)
            .send()
            .unwrap();
    }

    let tx_createPool_result =
        utils::handlers::handle_create_pool(&mut svm, &program_id, &admin, &token0, &token1);
    assert!(
        tx_createPool_result.is_ok(),
        "Create transaction failed: {:?}",
        tx_createPool_result.err()
    );

    let (pool_pda, _bump_pool_pda) =
        utils::get_utils::get_pool_pda(&program_id, &admin.pubkey(), &token0, &token1);
    let account_pool = svm.get_account(&pool_pda).expect("Should exist");

    let data = Pool::deserialize(&mut &account_pool.data[8..]).expect("Failed to deserialize Pool");

    println!("Pool exists: {}", svm.get_account(&pool_pda).is_some());
    assert_eq!(data.liquidity, 0);
    assert_eq!(data.token0, token0);
    assert_eq!(data.token1, token1);

    let (tokenliq_pda, tokenliq_bump) =
        utils::get_utils::get_tokenliq_pda(&program_id, &admin.pubkey(), &token0, &token1);

    let depositor_token0_ata_account = CreateAssociatedTokenAccount::new(&mut svm, &admin, &token0)
        .owner(&depositor.pubkey())
        .send()
        .unwrap();

    let depositor_token1_ata_account = CreateAssociatedTokenAccount::new(&mut svm, &admin, &token1)
        .owner(&depositor.pubkey())
        .send()
        .unwrap();

    let depositor_tokenliq_ata_account =
        CreateAssociatedTokenAccount::new(&mut svm, &admin, &tokenliq_pda)
            .owner(&depositor.pubkey())
            .send()
            .unwrap();

    let amount_a_to_mint = 10_000_000_000;
    let amount_b_to_mint = 10_000_000_000;

    // kurwa w dokumentacji pisze payer a to powinien byc mint authority!!!!
    // do napisania w dyskusji jebanej
    MintTo::new(
        &mut svm,
        &admin,
        &token0,
        &depositor_token0_ata_account,
        amount_a_to_mint,
    )
    .send()
    .unwrap();

    MintTo::new(
        &mut svm,
        &admin,
        &token1,
        &depositor_token1_ata_account,
        amount_b_to_mint,
    )
    .send()
    .unwrap();

    let amount_a = 10_000_000;
    let amount_b = 10_000_000;
    let tx_deposit_result = utils::handlers::handle_deposit(
        &mut svm,
        &program_id,
        &admin,
        &depositor,
        &token0,
        &token1,
        &amount_a,
        &amount_b,
    );
    let account_ata = svm
        .get_account(&depositor_tokenliq_ata_account)
        .expect("Should exist");
    let amount = Account::unpack(&account_ata.data)
        .expect("failed to unpack token")
        .amount;

    let depositor_ata_token0_address =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &depositor.pubkey(),
            &token0,
            &spl_token::id(),
        );
    let depositor_ata_token0 = svm
        .get_account(&depositor_ata_token0_address)
        .expect("Should exist");
    let depositor_token0_amount = Account::unpack(&depositor_ata_token0.data)
        .expect("failed to unpack token")
        .amount;

    let depositor_ata_token1_address =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &depositor.pubkey(),
            &token1,
            &spl_token::id(),
        );
    let depositor_ata_token1 = svm
        .get_account(&depositor_ata_token1_address)
        .expect("Should exist");
    let depositor_token1_amount = Account::unpack(&depositor_ata_token1.data)
        .expect("failed to unpack token")
        .amount;

    let expected_token0_after_deposit = amount_a_to_mint - amount_a;
    let expected_token1_after_deposit = amount_b_to_mint - amount_b;

    let expected_liquidity = utils::get_utils::get_expected_liquidity(&amount_a, &amount_b);

    assert_eq!(expected_liquidity.unwrap(), amount);
    assert_eq!(expected_token0_after_deposit, depositor_token0_amount);
    assert_eq!(expected_token1_after_deposit, depositor_token1_amount);
}

#[test]
fn test_revert_initialization_attack() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let depositor = Keypair::new();

    let program_keypair = read_keypair_file("../../target/deploy/amm-keypair.json")
        .expect("Failed to read program keypair");
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../target/deploy/amm.so");
    svm.add_program(program_id, program_bytes);

    svm.airdrop(&admin.pubkey(), 1000_000_000).unwrap();
    svm.airdrop(&depositor.pubkey(), 1_000_000_000_000).unwrap();

    // First we set admin
    let ix = utils::build_utils::build_initAdmin_instruction(&program_id, &admin.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin.pubkey()),
        &[&admin],
        svm.latest_blockhash(),
    );

    let tx_result = svm.send_transaction(tx);
    assert!(
        tx_result.is_ok(),
        "Create transaction failed: {:?}",
        tx_result.err()
    );

    let token0 = CreateMint::new(&mut svm, &admin)
        .authority(&admin.pubkey())
        .decimals(DECIMALS)
        .send()
        .unwrap();

    let mut token1 = CreateMint::new(&mut svm, &admin)
        .authority(&admin.pubkey())
        .decimals(DECIMALS)
        .send()
        .unwrap();

    while token1 <= token0 {
        token1 = CreateMint::new(&mut svm, &admin)
            .authority(&admin.pubkey())
            .decimals(DECIMALS)
            .send()
            .unwrap();
    }

    let ix_createPool = utils::build_utils::build_createPool_instruction(
        &program_id,
        &admin.pubkey(),
        &token0,
        &token1,
    );
    let tx_createPool = Transaction::new_signed_with_payer(
        &[ix_createPool],
        Some(&admin.pubkey()),
        &[&admin],
        svm.latest_blockhash(),
    );

    let tx_result = svm.send_transaction(tx_createPool);
    assert!(
        tx_result.is_ok(),
        "Create transaction failed: {:?}",
        tx_result.err()
    );

    let (pool_pda, _bump_pool_pda) =
        utils::get_utils::get_pool_pda(&program_id, &admin.pubkey(), &token0, &token1);
    let account_pool = svm.get_account(&pool_pda).expect("Should exist");

    let data = Pool::deserialize(&mut &account_pool.data[8..]).expect("Failed to deserialize Pool");

    assert_eq!(data.liquidity, 0);
    assert_eq!(data.token0, token0);
    assert_eq!(data.token1, token1);

    let (tokenliq_pda, tokenliq_bump) =
        utils::get_utils::get_tokenliq_pda(&program_id, &admin.pubkey(), &token0, &token1);

    let depositor_token0_ata_account = CreateAssociatedTokenAccount::new(&mut svm, &admin, &token0)
        .owner(&depositor.pubkey())
        .send()
        .unwrap();

    let depositor_token1_ata_account = CreateAssociatedTokenAccount::new(&mut svm, &admin, &token1)
        .owner(&depositor.pubkey())
        .send()
        .unwrap();

    let depositor_tokenliq_ata_account =
        CreateAssociatedTokenAccount::new(&mut svm, &admin, &tokenliq_pda)
            .owner(&depositor.pubkey())
            .send()
            .unwrap();

    let amount_a_to_mint = 10_000_000_000;
    let amount_b_to_mint = 10_000_000_000;

    // kurwa w dokumentacji pisze payer a to powinien byc mint authority!!!!
    // do napisania w dyskusji jebanej
    MintTo::new(
        &mut svm,
        &admin,
        &token0,
        &depositor_token0_ata_account,
        amount_a_to_mint,
    )
    .send()
    .unwrap();

    MintTo::new(
        &mut svm,
        &admin,
        &token1,
        &depositor_token1_ata_account,
        amount_b_to_mint,
    )
    .send()
    .unwrap();

    let amount_a = 10_000_000;
    let amount_b = 10_000_000;
    let ix_deposit = utils::build_utils::build_deposit_instruction(
        &program_id,
        &admin.pubkey(),
        &depositor.pubkey(),
        &token0,
        &token1,
        &amount_a,
        &amount_b,
    );
    let tx_deposit = Transaction::new_signed_with_payer(
        &[ix_deposit],
        Some(&depositor.pubkey()),
        &[&depositor],
        svm.latest_blockhash(),
    );

    let tx_deposit_result = svm.send_transaction(tx_deposit).unwrap();

    let account_ata = svm
        .get_account(&depositor_tokenliq_ata_account)
        .expect("Should exist");
    let amount = Account::unpack(&account_ata.data)
        .expect("failed to unpack token")
        .amount;

    let depositor_ata_token0_address =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &depositor.pubkey(),
            &token0,
            &spl_token::id(),
        );
    let depositor_ata_token0 = svm
        .get_account(&depositor_ata_token0_address)
        .expect("Should exist");
    let depositor_token0_amount = Account::unpack(&depositor_ata_token0.data)
        .expect("failed to unpack token")
        .amount;

    let depositor_ata_token1_address =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &depositor.pubkey(),
            &token1,
            &spl_token::id(),
        );
    let depositor_ata_token1 = svm
        .get_account(&depositor_ata_token1_address)
        .expect("Should exist");
    let depositor_token1_amount = Account::unpack(&depositor_ata_token1.data)
        .expect("failed to unpack token")
        .amount;

    let expected_token0_after_deposit = amount_a_to_mint - amount_a;
    let expected_token1_after_deposit = amount_b_to_mint - amount_b;

    let expected_liquidity = utils::get_utils::get_expected_liquidity(&amount_a, &amount_b);

    assert_eq!(expected_liquidity.unwrap(), amount);
    assert_eq!(expected_token0_after_deposit, depositor_token0_amount);
    assert_eq!(expected_token1_after_deposit, depositor_token1_amount);
}
