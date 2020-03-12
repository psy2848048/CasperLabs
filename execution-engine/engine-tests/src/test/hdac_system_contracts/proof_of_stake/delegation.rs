use num_traits::identities::Zero;

use engine_core::engine_state::{
    genesis::{GenesisAccount, POS_BONDING_PURSE},
    CONV_RATE,
};
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder},
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{
    account::{PublicKey, PurseId},
    Key, U512,
};

const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];
const ACCOUNT_2_ADDR: [u8; 32] = [2u8; 32];
const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
const ACCOUNT_2_DELEGATE_AMOUNT: u64 = 32_000;
const ACCOUNT_2_UNDELEGATE_AMOUNT: u64 = 20_000;

const CONTRACT_POS_DELEGATION: &str = "pos_delegation.wasm";

const DELEGATE_METHOD: &str = "delegate";
const UNDELEGATE_METHOD: &str = "undelegate";
const _REDELEGATE_METHOD: &str = "redelegate";

fn get_pos_bonding_purse_balance(builder: &InMemoryWasmTestBuilder) -> U512 {
    let purse_id = builder
        .get_pos_contract()
        .named_keys()
        .get(POS_BONDING_PURSE)
        .and_then(Key::as_uref)
        .map(|u| PurseId::new(*u))
        .expect("should find PoS payment purse");

    builder.get_purse_balance(purse_id)
}

#[ignore]
#[test]
fn should_run_successful_delegate_and_undelegate() {
    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2: a not bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_1_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_2_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts))
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be a genesis self-delegation
    let lookup_key = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR),
        base16::encode_lower(&ACCOUNT_1_ADDR),
        GENESIS_VALIDATOR_STAKE
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // execute delegate
    // delegate from ACCOUNT_2_ADDR to ACCOUNT_1_ADDR with 32k(ACCOUNT_2_DELEGATE_AMOUNT)
    let delegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(DELEGATE_METHOD),
            PublicKey::new(ACCOUNT_1_ADDR),
            U512::from(ACCOUNT_2_DELEGATE_AMOUNT),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(delegate_request)
        .expect_success()
        .commit()
        .finish();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be a still only one validator.
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("v_"))
            .count(),
        1
    );

    // that validator should be v_{ACCOUNT_1}_{GENESIS_VALIDATOR_STAKE + ACCOUNT_2_DELEGATE_AMOUNT}
    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR),
        GENESIS_VALIDATOR_STAKE + ACCOUNT_2_DELEGATE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // there should be 2 delegations
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("d_"))
            .count(),
        2
    );

    // there should be d_{ACCOUNT_2}_{ACCOUNT_1}_{ACCOUNT_2_DELEGATE_AMOUNT}
    let lookup_key = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_2_ADDR),
        base16::encode_lower(&ACCOUNT_1_ADDR),
        ACCOUNT_2_DELEGATE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // validate pos_bonding_purse balance
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(GENESIS_VALIDATOR_STAKE + ACCOUNT_2_DELEGATE_AMOUNT)
    );

    // validate ACCOUNT_2's balance
    let delegate_response = builder
        .get_exec_response(0)
        .expect("should have exec response");
    let gas_cost = utils::get_exec_costs(delegate_response)[0];

    let account_2 = builder
        .get_account(ACCOUNT_2_ADDR)
        .expect("should get account 2");
    assert_eq!(
        result.builder().get_purse_balance(account_2.purse_id()),
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE - ACCOUNT_2_DELEGATE_AMOUNT)
            - Motes::from_gas(gas_cost, CONV_RATE)
                .expect("should convert")
                .value()
    );

    // execute undelegate
    // undelegate {ACCOUNT_2}_{ACCOUNT_1}_{ACCOUNT_2_UNDELEGATE_AMOUNT}
    let undelegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(UNDELEGATE_METHOD),
            PublicKey::new(ACCOUNT_1_ADDR),
            Some(U512::from(ACCOUNT_2_UNDELEGATE_AMOUNT)),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(undelegate_request)
        .expect_success()
        .commit()
        .finish();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // validate validator stake amount
    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR),
        GENESIS_VALIDATOR_STAKE + ACCOUNT_2_DELEGATE_AMOUNT - ACCOUNT_2_UNDELEGATE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // there should be still 2 delegations
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("d_"))
            .count(),
        2
    );

    // validate delegation amount which is deducted with ACCOUNT_2_UNDELEGATE_AMOUNT.
    let lookup_key = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_2_ADDR),
        base16::encode_lower(&ACCOUNT_1_ADDR),
        ACCOUNT_2_DELEGATE_AMOUNT - ACCOUNT_2_UNDELEGATE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // validate pos_bonding_purse balance
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(
            GENESIS_VALIDATOR_STAKE + ACCOUNT_2_DELEGATE_AMOUNT - ACCOUNT_2_UNDELEGATE_AMOUNT
        )
    );

    // validate ACCOUNT_2's balance
    let undelegate_response = builder
        .get_exec_response(0)
        .expect("should have exec response");
    // gas cost of (delegate_request + undelegate_request)
    let gas_cost = gas_cost + utils::get_exec_costs(undelegate_response)[0];

    let account_2 = builder
        .get_account(ACCOUNT_2_ADDR)
        .expect("should get account 2");
    assert_eq!(
        result.builder().get_purse_balance(account_2.purse_id()),
        U512::from(
            DEFAULT_ACCOUNT_INITIAL_BALANCE - ACCOUNT_2_DELEGATE_AMOUNT
                + ACCOUNT_2_UNDELEGATE_AMOUNT
        ) - Motes::from_gas(gas_cost, CONV_RATE)
            .expect("should convert")
            .value()
    );

    // execute undelegate all with None
    // undelegate {ACCOUNT_2}_{ACCOUNT_1} all
    let undelegate_all_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(UNDELEGATE_METHOD),
            PublicKey::new(ACCOUNT_1_ADDR),
            None as Option<U512>,
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(undelegate_all_request)
        .expect_success()
        .commit()
        .finish();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // validate validator stake amount
    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR),
        GENESIS_VALIDATOR_STAKE
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // there should be only one delegation
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("d_"))
            .count(),
        1
    );

    // there should be no delegation starting with d_{ACCOUNT_2}
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(
                |(key, _)| key.starts_with(&format!("d_{}", base16::encode_lower(&ACCOUNT_2_ADDR)))
            )
            .count(),
        0
    );

    // validate pos_bonding_purse balance
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(GENESIS_VALIDATOR_STAKE)
    );

    // validate ACCOUNT_2's balance
    let undelegate_all_response = builder
        .get_exec_response(0)
        .expect("should have exec response");
    // gas cost of (delegate_request + undelegate_request + undelegate_all_request)
    let gas_cost = gas_cost + utils::get_exec_costs(undelegate_all_response)[0];

    let account_2 = builder
        .get_account(ACCOUNT_2_ADDR)
        .expect("should get account 2");
    assert_eq!(
        result.builder().get_purse_balance(account_2.purse_id()),
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)
            - Motes::from_gas(gas_cost, CONV_RATE)
                .expect("should convert")
                .value()
    );
}

#[ignore]
#[test]
fn should_run_successful_redelegate() {}

#[ignore]
#[test]
fn should_fail_to_unbond_more_than_own_self_delegation() {}

#[ignore]
#[test]
fn should_fail_to_delegate_to_unbonded_validator() {}

#[ignore]
#[test]
fn should_fail_to_redelegate_non_existent_delegation() {}

#[ignore]
#[test]
fn should_fail_to_self_redelegate() {}

#[ignore]
#[test]
fn should_fail_to_redelegate_more_than_own_shares() {}

#[ignore]
#[test]
fn should_fail_to_undelegate_with_insufficient_amount() {}

#[ignore]
#[test]
fn should_fail_to_delegate_with_insufficient_amount() {}
