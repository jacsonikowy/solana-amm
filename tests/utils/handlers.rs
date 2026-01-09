use {
    litesvm::{types::TransactionResult, LiteSVM},
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
};

use crate::utils::build_utils;
use crate::utils::get_utils;

pub fn handle_init_admin(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    admin: &Keypair,
) -> TransactionResult {
    let ix = build_utils::build_initAdmin_instruction(&program_id, &admin.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin.pubkey()),
        &[&admin],
        svm.latest_blockhash(),
    );

    let tx_result = svm.send_transaction(tx);
    tx_result
}

pub fn handle_set_admin(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    new_admin: &Keypair,
    admin: &Keypair,
) -> TransactionResult {
    let ix =
        build_utils::build_setAdmin_instruction(&program_id, &admin.pubkey(), &new_admin.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin.pubkey()),
        &[&admin],
        svm.latest_blockhash(),
    );

    let tx_result = svm.send_transaction(tx);
    tx_result
}

pub fn handle_create_pool(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    admin: &Keypair,
    token0: &Pubkey,
    token1: &Pubkey,
) -> TransactionResult {
    let ix =
        build_utils::build_createPool_instruction(&program_id, &admin.pubkey(), &token0, &token1);
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&admin.pubkey()),
        &[&admin],
        svm.latest_blockhash(),
    );

    let tx_result = svm.send_transaction(tx);
    tx_result
}

pub fn handle_deposit(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    admin: &Keypair,
    depositor: &Keypair,
    token0: &Pubkey,
    token1: &Pubkey,
    amount_a: &u64,
    amount_b: &u64,
) -> TransactionResult {
    let ix = build_utils::build_deposit_instruction(
        &program_id,
        &admin.pubkey(),
        &depositor.pubkey(),
        &token0,
        &token1,
        &amount_a,
        &amount_b,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&depositor.pubkey()),
        &[&depositor],
        svm.latest_blockhash(),
    );

    let tx_result = svm.send_transaction(tx);
    tx_result
}

pub fn handle_withdraw(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    admin: &Keypair,
    depositor: &Keypair,
    token0: &Pubkey,
    token1: &Pubkey,
    amount: &u64,
) -> TransactionResult {
    let ix = build_utils::build_withdraw_instruction(
        &program_id,
        &admin.pubkey(),
        &depositor.pubkey(),
        &token0,
        &token1,
        &amount,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&depositor.pubkey()),
        &[&depositor],
        svm.latest_blockhash(),
    );
    let tx_result = svm.send_transaction(tx);
    tx_result
}

pub fn handle_swapExactInput(
    svm: &mut LiteSVM,
    program_id: &Pubkey,
    admin: &Keypair,
    swapper: &Keypair,
    token0: &Pubkey,
    token1: &Pubkey,
    token_in: &Pubkey,
    amount: &u64,
) -> TransactionResult {
    let ix = build_utils::build_swapExactInput_instruction(
        &program_id,
        &admin.pubkey(),
        &swapper.pubkey(),
        &token0,
        &token1,
        &token_in,
        &amount,
    );
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&swapper.pubkey()),
        &[&swapper],
        svm.latest_blockhash(),
    );
    let tx_result = svm.send_transaction(tx);
    tx_result
}
