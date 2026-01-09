use borsh::BorshDeserialize;
use litesvm::LiteSVM;
use litesvm_token::{spl_token::native_mint::DECIMALS, CreateAssociatedTokenAccount, CreateMint};
use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use solana_system_interface::program::ID as system_program_id;
use spl_associated_token_account::get_associated_token_address;

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
fn test_create_pool() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let program_keypair = read_keypair_file("../../target/deploy/amm-keypair.json")
        .expect("Failed to read program keypair");
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../target/deploy/amm.so");
    svm.add_program(program_id, program_bytes);

    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();

    helper_set_admin(&program_id, &admin, &admin.pubkey(), &mut svm);

    // consider adding also this to helper
    let (admin_settings, _bump) = utils::get_utils::get_admin_settings_pda(&program_id);
    let account_admin_settings = svm.get_account(&admin_settings).expect("Should exist");

    let data = AdminSettings::deserialize(&mut &account_admin_settings.data[8..])
        .expect("Failed to deserialize AdminSettings");

    assert_eq!(data.admin, admin.pubkey(), "Owner mismatch");

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

    let tx_result =
        utils::handlers::handle_create_pool(&mut svm, &program_id, &admin, &token0, &token1);
    assert!(
        tx_result.is_ok(),
        "Create transaction faile: {:?}",
        tx_result.err()
    );

    let (pool_pda, _bump_pool_pda) =
        utils::get_utils::get_pool_pda(&program_id, &admin.pubkey(), &token0, &token1);
    let account_pool = svm.get_account(&pool_pda).expect("Should exist");

    let data = Pool::deserialize(&mut &account_pool.data[8..]).expect("Failed to deserialize Pool");

    assert_eq!(data.liquidity, 0);
    assert_eq!(data.token0, token0);
    assert_eq!(data.token1, token1);
}

fn helper_set_admin(program_id: &Pubkey, admin: &Keypair, admin_pub: &Pubkey, svm: &mut LiteSVM) {
    let ix = utils::build_utils::build_initAdmin_instruction(program_id, admin_pub);
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(admin_pub),
        &[admin],
        svm.latest_blockhash(),
    );

    let tx_result = svm.send_transaction(tx);
}
