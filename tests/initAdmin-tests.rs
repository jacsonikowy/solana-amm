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
fn test_init_admin() {
    let mut svm = LiteSVM::new();
    let admin = Keypair::new();
    let program_keypair = read_keypair_file("../../target/deploy/amm-keypair.json")
        .expect("Failed to read program keypair");
    let program_id = program_keypair.pubkey();
    let program_bytes = include_bytes!("../target/deploy/amm.so");
    svm.add_program(program_id, program_bytes);

    svm.airdrop(&admin.pubkey(), 10_000_000).unwrap();

    let tx_result = utils::handlers::handle_init_admin(&mut svm, &program_id, &admin);
    assert!(
        tx_result.is_ok(),
        "Create transaction failed: {:?}",
        tx_result.err()
    );

    let (admin_settings, _bump) = utils::get_utils::get_admin_settings_pda(&program_id);
    let account_admin_settings = svm.get_account(&admin_settings).expect("Should exist");

    let data = AdminSettings::deserialize(&mut &account_admin_settings.data[8..])
        .expect("Failed to deserialize AdminSettings");

    assert_eq!(data.admin, admin.pubkey(), "Owner mismatch");
}

#[test]
fn test_already_initialized() {
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

    let (admin_settings, _bump) = utils::get_utils::get_admin_settings_pda(&program_id);
    let account_admin_settings = svm.get_account(&admin_settings).expect("Should exist");

    let data = AdminSettings::deserialize(&mut &account_admin_settings.data[8..])
        .expect("Failed to deserialize AdminSettings");

    assert_eq!(data.admin, admin.pubkey(), "Owner mismatch");

    let tx_result_user = utils::handlers::handle_init_admin(&mut svm, &program_id, &user);
    assert!(tx_result_user.is_err());

    let err = tx_result_user.unwrap_err();
    match err.err {
        TransactionError::InstructionError(0, _) => {
            println!("Got expected error: InstructionError (account already in use)");
        }
        _ => panic!("Got unexpected error: {:?}", err.err),
    }
}
