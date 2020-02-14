use base16;

use contract_ffi::{
    key::Key,
    value::{account::PublicKey, U512},
};
use engine_core::engine_state::{genesis::GenesisAccount, SYSTEM_ACCOUNT_ADDR};
use engine_shared::{motes::Motes, stored_value::StoredValue, transform::Transform};

use crate::{
    support::test_support::{self, ExecuteRequestBuilder, InMemoryWasmTestBuilder},
    test::{DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG},
};

const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];

const TRANSFER_TO_ACCOUNT_METHOD: &str = "transfer_to_account";
const BOND_METHOD: &str = "bond";
const UNBOND_METHOD: &str = "unbond";

fn get_client_api_proxy_hash(builder: &InMemoryWasmTestBuilder) -> [u8; 32] {
    // query client_api_proxy_hash from SYSTEM_ACCOUNT
    let system_account = match builder
        .query(None, Key::Account(SYSTEM_ACCOUNT_ADDR), &[])
        .expect("should query system account")
    {
        StoredValue::Account(account) => account,
        _ => panic!("should get an account"),
    };

    system_account
        .named_keys()
        .get("client_api_proxy")
        .expect("should get client_api_proxy key")
        .as_hash()
        .expect("should be hash")
}

#[ignore]
#[test]
fn should_invoke_successful_transfer_to_account() {
    const TRANSFER_AMOUNT: u64 = 1000;

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_GENESIS_CONFIG).commit();

    let client_api_proxy_hash = get_client_api_proxy_hash(&builder);

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

#[ignore]
#[test]
fn should_invoke_successful_bond_and_unbond() {
    const BOND_AMOUNT: u64 = 1_000_000;

    // only DEFAULT_ACCOUNT is in initial validator queue.
    let accounts: Vec<GenesisAccount> = vec![GenesisAccount::new(
        PublicKey::new(DEFAULT_ACCOUNT_ADDR),
        Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
        Motes::new(BOND_AMOUNT.into()),
    )];
    let genesis_config = test_support::create_genesis_config(accounts);
    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&genesis_config)
        .commit()
        .finish();

    let client_api_proxy_hash = get_client_api_proxy_hash(result.builder());

    // Transfer to ACCOUNT_1_ADDR request
    let exec_request_transfer = ExecuteRequestBuilder::contract_call_by_hash(
        DEFAULT_ACCOUNT_ADDR,
        client_api_proxy_hash,
        (
            TRANSFER_TO_ACCOUNT_METHOD,
            ACCOUNT_1_ADDR,
            1_000_000_000 as u64,
        ),
    )
    .build();
    // Bonding request
    let exec_request_bonding = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR,
        client_api_proxy_hash,
        (BOND_METHOD, BOND_AMOUNT),
    )
    .build();
    let bonding_result = InMemoryWasmTestBuilder::from_result(result)
        .exec(exec_request_transfer)
        .expect_success()
        .commit()
        .exec(exec_request_bonding)
        .expect_success()
        .commit()
        .finish();

    let pos_uref = bonding_result.builder().get_pos_contract_uref();

    // retrieve transform of bond request.
    let transforms = &bonding_result.builder().get_transforms()[1];
    let pos_transform = &transforms[&Key::from(pos_uref).normalize()];

    let pos_contract = if let Transform::Write(StoredValue::Contract(contract)) = pos_transform {
        contract
    } else {
        panic!(
            "pos transform is expected to be of AddKeys variant but received {:?}",
            pos_transform
        );
    };

    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR),
        BOND_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // Unbonding request
    let exec_request_unbonding = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR,
        client_api_proxy_hash,
        (UNBOND_METHOD, None as Option<u64>), // None means unbond all the amount
    )
    .build();
    let unbonding_result = InMemoryWasmTestBuilder::from_result(bonding_result)
        .exec(exec_request_unbonding)
        .expect_success()
        .commit()
        .finish();

    let transforms = &unbonding_result.builder().get_transforms()[0];
    let pos_transform = &transforms[&Key::from(pos_uref).normalize()];

    let pos_contract = if let Transform::Write(StoredValue::Contract(contract)) = pos_transform {
        contract
    } else {
        panic!(
            "pos transform is expected to be of AddKeys variant but received {:?}",
            pos_transform
        );
    };

    // ensure that ACCOUNT_1_ADDR is not in validator queue.
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(
                |(key, _)| key.starts_with(&format!("v_{}", base16::encode_lower(&ACCOUNT_1_ADDR)))
            )
            .count(),
        0
    );
    // only genesis validator is still in the queue
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("v_"))
            .count(),
        1
    );
}
