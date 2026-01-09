use {
    borsh::BorshDeserialize,
    litesvm::LiteSVM,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
};

mod utils;

#[derive(Debug, BorshDeserialize)]
struct AdminSettings {
    pub admin: Pubkey,
}

#[test]
fn test_set_admin() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let new_admin = Keypair::new();
    let program_keypair = read_keypair_file("../../target/deploy/amm-keypair.json")
        .expect("Failed to read program keypair");
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../target/deploy/amm.so");
    svm.add_program(program_id, program_bytes);

    svm.airdrop(&admin.pubkey(), 10_000_000).unwrap();
    svm.airdrop(&new_admin.pubkey(), 10_000_000).unwrap();

    // First we initialize admin account

    let tx_result = utils::handlers::handle_init_admin(&mut svm, &program_id, &admin);
    assert!(
        tx_result.is_ok(),
        "Create transaction failed: {:?}",
        tx_result.err()
    );

    // Then we check for actual set
    let tx_result_set =
        utils::handlers::handle_set_admin(&mut svm, &program_id, &new_admin, &admin);
    assert!(
        tx_result_set.is_ok(),
        "Set admin tx failed: {:?}",
        tx_result_set.err()
    );

    let (admin_settings, _bump) = utils::get_utils::get_admin_settings_pda(&program_id);
    let account_admin_settings = svm.get_account(&admin_settings).expect("Should exist");

    let data = AdminSettings::deserialize(&mut &account_admin_settings.data[8..])
        .expect("Failed to deserialize AdminSettings");

    println!("{}", new_admin.pubkey());
    println!("{}", admin.pubkey());
    assert_eq!(data.admin, new_admin.pubkey(), "Owner mismatch");
}

#[test]
fn test_unauthorized_set() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let user = Keypair::new();

    let program_keypair = read_keypair_file("../../target/deploy/amm-keypair.json")
        .expect("Failed to read program keypair");
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../target/deploy/amm.so");
    svm.add_program(program_id, program_bytes);

    svm.airdrop(&admin.pubkey(), 10_000_000).unwrap();
    svm.airdrop(&user.pubkey(), 10_000_000).unwrap();

    let tx_result = utils::handlers::handle_init_admin(&mut svm, &program_id, &admin);
    assert!(
        tx_result.is_ok(),
        "Create transaction failed: {:?}",
        tx_result.err()
    );

    // Then we check for actual set
    let tx_result_set = utils::handlers::handle_set_admin(&mut svm, &program_id, &user, &user);
    assert!(tx_result_set.is_err());
}
