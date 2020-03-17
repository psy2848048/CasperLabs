use num_traits::identities::Zero;

use engine_core::engine_state::{
    genesis::{GenesisAccount, POS_BONDING_PURSE},
    CONV_RATE, SYSTEM_ACCOUNT_ADDR,
};
use engine_shared::{motes::Motes, stored_value::StoredValue, transform::Transform};
use types::{
    account::{PublicKey, PurseId},
    Key, U512,
};

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
const DELEGATE_METHOD: &str = "delegate";
const UNDELEGATE_METHOD: &str = "undelegate";
const REDELEGATE_METHOD: &str = "redelegate";

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

#[ignore]
#[test]
fn should_invoke_successful_delegation_methods() {
    const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];
    const ACCOUNT_2_ADDR: [u8; 32] = [2u8; 32];
    const ACCOUNT_3_ADDR: [u8; 32] = [3u8; 32];

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;

    const ACCOUNT_3_DELEGATE_AMOUNT: u64 = 32_000;
    const ACCOUNT_3_REDELEGATE_AMOUNT: u64 = 20_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2  a bonded account with the initial balance.
    // ACCOUNT_3: a not bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_1_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_2_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_3_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    let genesis_config = utils::create_genesis_config(accounts);
    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&genesis_config)
        .commit()
        .finish();

    let client_api_proxy_hash = get_client_api_proxy_hash(result.builder());

    // ACCOUNT_3 delegate to ACCOUNT_1
    let delegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR,
        client_api_proxy_hash,
        (
            DELEGATE_METHOD,
            PublicKey::new(ACCOUNT_1_ADDR),
            U512::from(ACCOUNT_3_DELEGATE_AMOUNT),
        ),
    )
    .build();

    // ACCOUNT_3 redelegate from ACCOUNT_1 to ACCOUNT_2
    let redelegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR,
        client_api_proxy_hash,
        (
            REDELEGATE_METHOD,
            PublicKey::new(ACCOUNT_1_ADDR),
            PublicKey::new(ACCOUNT_2_ADDR),
            U512::from(ACCOUNT_3_REDELEGATE_AMOUNT),
        ),
    )
    .build();

    // ACCOUNT_3 undelegate all from ACCOUNT_1
    let undelegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR,
        client_api_proxy_hash,
        (
            UNDELEGATE_METHOD,
            PublicKey::new(ACCOUNT_1_ADDR),
            None as Option<U512>,
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    builder
        .exec(delegate_request)
        .expect_success()
        .commit()
        .exec(redelegate_request)
        .expect_success()
        .commit()
        .expect_success()
        .exec(undelegate_request)
        .expect_success()
        .commit()
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("must get pos contract");

    // Validate delegations
    let expected_delegation_1 = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_3_ADDR),
        base16::encode_lower(&ACCOUNT_2_ADDR),
        ACCOUNT_3_REDELEGATE_AMOUNT
    );
    let delegation_key_that_should_not_exist = format!(
        "d_{}_{}",
        base16::encode_lower(&ACCOUNT_3_ADDR),
        base16::encode_lower(&ACCOUNT_1_ADDR)
    );
    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_delegation_1));
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with(&delegation_key_that_should_not_exist))
            .count(),
        0
    );
    // There are 2 self delegations and one delegation d_{ACCOUNT_3}_{ACCOUNT_2}_{REDELEGATE_AMOUNT}
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("d_"))
            .count(),
        3
    );

    // Validate stakes
    let expected_stakes_1 = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR),
        GENESIS_VALIDATOR_STAKE
    );
    let expected_stakes_2 = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_2_ADDR),
        GENESIS_VALIDATOR_STAKE + ACCOUNT_3_REDELEGATE_AMOUNT
    );

    assert!(pos_contract.named_keys().contains_key(&expected_stakes_1));
    assert!(pos_contract.named_keys().contains_key(&expected_stakes_2));

    // There should be only 2 stakes.
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("v_"))
            .count(),
        2
    );

    // Validate pos_bonding_purse balance
    let pos_bonding_purse_balance = {
        let purse_id = pos_contract
            .named_keys()
            .get(POS_BONDING_PURSE)
            .and_then(Key::as_uref)
            .map(|u| PurseId::new(*u))
            .expect("should find PoS payment purse");

        builder.get_purse_balance(purse_id)
    };
    assert_eq!(
        pos_bonding_purse_balance,
        (GENESIS_VALIDATOR_STAKE * 2 + ACCOUNT_3_REDELEGATE_AMOUNT).into()
    );
}