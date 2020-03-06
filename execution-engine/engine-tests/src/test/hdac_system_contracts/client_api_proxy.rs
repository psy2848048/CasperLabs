use engine_core::engine_state::{genesis::GenesisAccount, CONV_RATE, SYSTEM_ACCOUNT_ADDR};
use engine_shared::{motes::Motes, stored_value::StoredValue, transform::Transform};
use types::{account::PublicKey, Key, U512};

use engine_test_support::{
    internal::{
        utils, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder,
        DEFAULT_ACCOUNT_KEY, DEFAULT_GENESIS_CONFIG, DEFAULT_PAYMENT,
    },
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
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
        .into_hash()
        .expect("should be hash")
}

#[ignore]
#[test]
fn should_invoke_successful_transfer_to_account() {
    let transferred_amount = U512::from(1000);

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_GENESIS_CONFIG).commit();

    let client_api_proxy_hash = get_client_api_proxy_hash(&builder);

    // transfer to ACCOUNT_1_ADDR with transferred_amount
    let exec_request = ExecuteRequestBuilder::contract_call_by_hash(
        DEFAULT_ACCOUNT_ADDR,
        client_api_proxy_hash,
        (
            TRANSFER_TO_ACCOUNT_METHOD,
            ACCOUNT_1_ADDR,
            transferred_amount,
        ),
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

    assert_eq!(balance, transferred_amount);
}

#[ignore]
#[test]
fn should_invoke_successful_standard_payment() {
    // run genesis
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_GENESIS_CONFIG).commit();

    let client_api_proxy_hash = get_client_api_proxy_hash(&builder);

    // transfer 1 from DEFAULT_ACCOUNT to ACCOUNT_1
    let transferred_amount = 1;
    let exec_request = {
        let deploy = DeployItemBuilder::new()
            .with_address(DEFAULT_ACCOUNT_ADDR)
            .with_deploy_hash([1; 32])
            .with_session_code(
                "transfer_purse_to_account.wasm",
                (
                    PublicKey::new(ACCOUNT_1_ADDR),
                    U512::from(transferred_amount),
                ),
            )
            .with_stored_payment_hash(
                client_api_proxy_hash.to_vec(),
                ("standard_payment", *DEFAULT_PAYMENT),
            )
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_KEY])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };
    let transfer_result = builder.exec_commit_finish(exec_request);
    let default_account = transfer_result
        .builder()
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should get genesis account");
    let modified_balance = transfer_result
        .builder()
        .get_purse_balance(default_account.purse_id());
    let initial_balance = U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE);

    assert_ne!(
        modified_balance, initial_balance,
        "balance should be less than initial balance"
    );

    let response = transfer_result
        .builder()
        .get_exec_response(0)
        .expect("there should be a response")
        .clone();

    let success_result = utils::get_success_result(&response);
    let fee_in_motes =
        Motes::from_gas(success_result.cost(), CONV_RATE).expect("should have motes");
    let total_consumed = fee_in_motes.value() + U512::from(transferred_amount);
    let tally = total_consumed + modified_balance;

    assert_eq!(
        initial_balance, tally,
        "no net resources should be gained or lost post-distribution"
    );
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
    let genesis_config = utils::create_genesis_config(accounts);
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
            *DEFAULT_PAYMENT * 5,
        ),
    )
    .build();
    // Bonding request
    let exec_request_bonding = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR,
        client_api_proxy_hash,
        (BOND_METHOD, U512::from(BOND_AMOUNT)),
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
        (UNBOND_METHOD, None as Option<U512>), // None means unbond all the amount
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
