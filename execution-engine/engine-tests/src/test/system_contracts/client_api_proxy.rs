use contract_ffi::{key::Key, value::U512};
use engine_core::engine_state::SYSTEM_ACCOUNT_ADDR;
use engine_shared::stored_value::StoredValue;

use crate::{
    support::test_support::{ExecuteRequestBuilder, InMemoryWasmTestBuilder},
    test::{DEFAULT_ACCOUNT_ADDR, DEFAULT_GENESIS_CONFIG},
};

const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];

#[ignore]
#[test]
fn should_invoke_successfully_transfer_to_account() {
    const TRANSFER_AMOUNT: u64 = 1000;
    const TRANSFER_TO_ACCOUNT_METHOD: &str = "transfer_to_account";

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_GENESIS_CONFIG).commit();

    // query client_api_proxy_hash from SYSTEM_ACCOUNT
    let system_account = match builder
        .query(None, Key::Account(SYSTEM_ACCOUNT_ADDR), &[])
        .expect("should query system account")
    {
        StoredValue::Account(account) => account,
        _ => panic!("should get an account"),
    };

    let client_api_proxy_hash = system_account
        .named_keys()
        .get("client_api_proxy")
        .expect("should get client_api_proxy key")
        .as_hash()
        .expect("should be hash");

    // transfer to ACCOUNT_1_ADDR with TRANSFER_AMOUNT
    let exec_request = ExecuteRequestBuilder::contract_call_by_hash(
        DEFAULT_ACCOUNT_ADDR,
        client_api_proxy_hash,
        (TRANSFER_TO_ACCOUNT_METHOD, ACCOUNT_1_ADDR, TRANSFER_AMOUNT),
    )
    .build();

    let test_result = builder.exec_commit_finish(exec_request);

    let account_1 = test_result
        .builder()
        .get_account(ACCOUNT_1_ADDR)
        .expect("should get account 1");

    let balance = test_result
        .builder()
        .get_purse_balance(account_1.purse_id());

    assert_eq!(balance, U512::from(TRANSFER_AMOUNT));
}
