use engine_core::engine_state::CONV_RATE;
use engine_test_support::{
    internal::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_CASPER_GENESIS_CONFIG,
        DEFAULT_PAYMENT,
    },
    DEFAULT_ACCOUNT_ADDR,
};
use types::U512;

const CONTRACT_POS_GET_PAYMENT_PURSE: &str = "pos_get_payment_purse.wasm";
const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";
const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];

#[ignore]
#[test]
fn should_run_get_payment_purse_contract_default_account() {
    let exec_request = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_GET_PAYMENT_PURSE,
        (*DEFAULT_PAYMENT,),
    )
    .build();
    InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_CASPER_GENESIS_CONFIG)
        .exec(exec_request)
        .expect_success()
        .commit();
}

#[ignore]
#[test]
fn should_run_get_payment_purse_contract_account_1() {
    let account_1_initial_balance: U512 = *DEFAULT_PAYMENT + 10 * CONV_RATE;

    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        (ACCOUNT_1_ADDR, account_1_initial_balance),
    )
    .build();
    let exec_request_2 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_GET_PAYMENT_PURSE,
        (*DEFAULT_PAYMENT,),
    )
    .build();
    InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_CASPER_GENESIS_CONFIG)
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .exec(exec_request_2)
        .expect_success()
        .commit();
}
