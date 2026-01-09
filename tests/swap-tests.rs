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
fn test_successful_swap() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let depositor = Keypair::new();

    let alice = Keypair::new();
    let bob = Keypair::new();

    let program_keypair = read_keypair_file("../../target/deploy/amm-keypair.json")
        .expect("Failed to read program keypair");
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../target/deploy/amm.so");
    svm.add_program(program_id, program_bytes);

    svm.airdrop(&admin.pubkey(), 1000_000_000).unwrap();
    svm.airdrop(&depositor.pubkey(), 1_000_000_000_000).unwrap();
    svm.airdrop(&alice.pubkey(), 1_000_000).unwrap();
    svm.airdrop(&bob.pubkey(), 1_000_000).unwrap();

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

    // maybe i should add to them init if needed but the thing is idk
    let alice_token0_ata_account = CreateAssociatedTokenAccount::new(&mut svm, &admin, &token0)
        .owner(&alice.pubkey())
        .send()
        .unwrap();

    let alice_token1_ata_account = CreateAssociatedTokenAccount::new(&mut svm, &admin, &token1)
        .owner(&alice.pubkey())
        .send()
        .unwrap();

    // We're minting for user tokens for a swap
    let alice_amount_a = 100;
    MintTo::new(
        &mut svm,
        &admin,
        &token0,
        &alice_token0_ata_account,
        alice_amount_a,
    )
    .send()
    .unwrap();

    let tx_swap_result = utils::handlers::handle_swapExactInput(
        &mut svm,
        &program_id,
        &admin,
        &alice,
        &token0,
        &token1,
        &token0,
        &alice_amount_a,
    )
    .unwrap();

    // check for ata 1 and ata 0 that on is bigger than the other one i think
    let alice_ata_token0 = svm
        .get_account(&alice_token0_ata_account)
        .expect("Should exist");
    let alice_token0_amount = Account::unpack(&alice_ata_token0.data)
        .expect("failed to unpack token")
        .amount;

    let alice_ata_token1 = svm
        .get_account(&alice_token1_ata_account)
        .expect("Should exist");
    let alice_token1_amount = Account::unpack(&alice_ata_token1.data)
        .expect("failed to unpack token")
        .amount;

    assert!(alice_token1_amount > alice_token0_amount);
}
