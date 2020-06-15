use num_traits::identities::Zero;

use engine_core::engine_state::genesis::GenesisAccount;
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, StepRequestBuilder},
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{account::PublicKey, ApiError, U512};

const CONTRACT_POS_DELEGATION: &str = "pos_delegation.wasm";

const BOND_METHOD: &str = "bond";
const UNBOND_METHOD: &str = "unbond";
const DELEGATE_METHOD: &str = "delegate";
const UNDELEGATE_METHOD: &str = "undelegate";
const REDELEGATE_METHOD: &str = "redelegate";

#[ignore]
#[test]
fn should_run_successful_delegate_and_undelegate() {
    const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR: PublicKey = PublicKey::ed25519_from([2u8; 32]);
    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_2_DELEGATE_AMOUNT: u64 = 32_000;
    const ACCOUNT_2_UNDELEGATE_AMOUNT: u64 = 20_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2: a not bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            ACCOUNT_1_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            ACCOUNT_2_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    let account_1_bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_DELEGATION,
        (String::from(BOND_METHOD), U512::from(1_000_000)),
    )
    .build();
    let account_2_bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (String::from(BOND_METHOD), U512::from(1_000_000)),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(account_1_bond_request)
        .expect_success()
        .commit()
        .exec(account_2_bond_request)
        .expect_success()
        .commit()
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be a genesis self-delegation
    let lookup_key = format!(
        "d_{}_{}_{}",
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
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
            ACCOUNT_1_ADDR,
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
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
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
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
        ACCOUNT_2_DELEGATE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // execute undelegate
    // undelegate {ACCOUNT_2}_{ACCOUNT_1}_{ACCOUNT_2_UNDELEGATE_AMOUNT}
    let undelegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(UNDELEGATE_METHOD),
            ACCOUNT_1_ADDR,
            Some(U512::from(ACCOUNT_2_UNDELEGATE_AMOUNT)),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(undelegate_request)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // validate validator stake amount
    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
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
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
        ACCOUNT_2_DELEGATE_AMOUNT - ACCOUNT_2_UNDELEGATE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // execute undelegate all with None
    // undelegate {ACCOUNT_2}_{ACCOUNT_1} all
    let undelegate_all_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(UNDELEGATE_METHOD),
            ACCOUNT_1_ADDR,
            None as Option<U512>,
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _ = builder
        .exec(undelegate_all_request)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // validate validator stake amount
    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
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
            .filter(|(key, _)| key.starts_with(&format!(
                "d_{}",
                base16::encode_lower(ACCOUNT_2_ADDR.as_bytes())
            )))
            .count(),
        0
    );
}

#[ignore]
#[test]
fn should_run_successful_redelegate() {
    const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR: PublicKey = PublicKey::ed25519_from([2u8; 32]);
    const ACCOUNT_3_ADDR: PublicKey = PublicKey::ed25519_from([3u8; 32]);

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_3_DELEGATE_AMOUNT: u64 = 32_000;
    const ACCOUNT_3_REDELEGATE_AMOUNT: u64 = 20_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2  a bonded account with the initial balance.
    // ACCOUNT_3: a not bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            ACCOUNT_1_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            ACCOUNT_2_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            ACCOUNT_3_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    let account_1_bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_DELEGATION,
        (String::from(BOND_METHOD), U512::from(1_000_000)),
    )
    .build();
    let account_2_bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (String::from(BOND_METHOD), U512::from(1_000_000)),
    )
    .build();
    let account_3_bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR,
        CONTRACT_POS_DELEGATION,
        (String::from(BOND_METHOD), U512::from(1_000_000)),
    )
    .build();

    // delegate request from ACCOUNT_3 to ACCOUNT_1.
    let delegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(DELEGATE_METHOD),
            ACCOUNT_1_ADDR,
            U512::from(ACCOUNT_3_DELEGATE_AMOUNT),
        ),
    )
    .build();
    // redelegate request from ACCOUNT_3 which redelegates from ACCOUNT_1 to ACCOUNT_2.
    let redelegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(REDELEGATE_METHOD),
            ACCOUNT_1_ADDR,
            ACCOUNT_2_ADDR,
            Some(U512::from(ACCOUNT_3_REDELEGATE_AMOUNT)),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(account_1_bond_request)
        .expect_success()
        .commit()
        .exec(account_2_bond_request)
        .expect_success()
        .commit()
        .exec(account_3_bond_request)
        .expect_success()
        .commit()
        .exec(delegate_request)
        .expect_success()
        .commit()
        .exec(redelegate_request)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // validate stakes
    let expected_account_1_stake = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
        GENESIS_VALIDATOR_STAKE + ACCOUNT_3_DELEGATE_AMOUNT - ACCOUNT_3_REDELEGATE_AMOUNT
    );
    let expected_account_2_stake = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        GENESIS_VALIDATOR_STAKE + ACCOUNT_3_REDELEGATE_AMOUNT
    );

    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_account_1_stake));
    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_account_2_stake));

    // validate delegations
    let expected_delegation_1 = format!(
        "d_{}_{}_{}",
        base16::encode_lower(ACCOUNT_3_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
        ACCOUNT_3_DELEGATE_AMOUNT - ACCOUNT_3_REDELEGATE_AMOUNT
    );
    let expected_delegation_2 = format!(
        "d_{}_{}_{}",
        base16::encode_lower(ACCOUNT_3_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        ACCOUNT_3_REDELEGATE_AMOUNT
    );
    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_delegation_1));
    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_delegation_2));

    // redelegate all request
    let redelegate_all_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(REDELEGATE_METHOD),
            ACCOUNT_1_ADDR,
            ACCOUNT_2_ADDR,
            Some(U512::from(
                ACCOUNT_3_DELEGATE_AMOUNT - ACCOUNT_3_REDELEGATE_AMOUNT,
            )),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _ = builder
        .exec(redelegate_all_request)
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // validate stakes
    let expected_account_1_stake = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
        GENESIS_VALIDATOR_STAKE
    );
    let expected_account_2_stake = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        GENESIS_VALIDATOR_STAKE + ACCOUNT_3_DELEGATE_AMOUNT
    );

    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_account_1_stake));
    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_account_2_stake));

    // validate delegations
    let expected_delegation = format!(
        "d_{}_{}_{}",
        base16::encode_lower(ACCOUNT_3_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        ACCOUNT_3_DELEGATE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&expected_delegation));

    // there should be only one delegation starting with d_{ACCOUNT_3}
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with(&format!(
                "d_{}",
                base16::encode_lower(ACCOUNT_3_ADDR.as_bytes())
            )))
            .count(),
        1
    );
}

#[ignore]
#[test]
fn should_fail_to_undelegate_more_than_delegation() {
    const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR: PublicKey = PublicKey::ed25519_from([2u8; 32]);

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_2_DELEGATE_AMOUNT: u64 = 32_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2: a not bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            ACCOUNT_1_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            ACCOUNT_2_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    // bond request from ACCOUNT_2.
    let bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(BOND_METHOD),
            U512::from(GENESIS_VALIDATOR_STAKE),
        ),
    )
    .build();

    // delegate request from ACCOUNT_2 to ACCOUNT_1.
    let delegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(DELEGATE_METHOD),
            ACCOUNT_1_ADDR,
            U512::from(ACCOUNT_2_DELEGATE_AMOUNT),
        ),
    )
    .build();

    let undelegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(UNBOND_METHOD),
            Some(U512::from(ACCOUNT_2_DELEGATE_AMOUNT + 10)),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let _ = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(bond_request)
        .expect_success()
        .commit()
        .exec(delegate_request)
        .expect_success()
        .commit()
        .exec(undelegate_request)
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    // assert that the delegations are not changed
    let delegation_1 = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_2_ADDR.value()),
        base16::encode_lower(&ACCOUNT_1_ADDR.value()),
        ACCOUNT_2_DELEGATE_AMOUNT
    );
    let pop_contract = builder.get_pos_contract();
    assert!(pop_contract.named_keys().contains_key(&delegation_1));

    // assert ACCOUNT_2's delegating amount
    // assert ACCOUNT_1's delegated amount
}

#[ignore]
#[test]
fn should_fail_to_delegate_to_not_self_delegated_validator() {
    const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR: PublicKey = PublicKey::ed25519_from([2u8; 32]);

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_1_DELEGATE_AMOUNT: u64 = 32_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2: a not bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            ACCOUNT_1_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            ACCOUNT_2_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    // delegate request from ACCOUNT_1 to ACCOUNT_2.
    let delegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(DELEGATE_METHOD),
            ACCOUNT_2_ADDR,
            U512::from(ACCOUNT_1_DELEGATE_AMOUNT),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(delegate_request)
        .commit()
        .finish();

    let response = result
        .builder()
        .get_exec_response(0)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);

    // pos::Error::NotSelfDelegated => 27
    assert!(error_message.contains(&format!(
        "Revert({})",
        u32::from(ApiError::ProofOfStake(27))
    )));
}

#[ignore]
#[test]
fn should_fail_to_redelegate_non_existent_delegation() {
    const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];
    const ACCOUNT_2_ADDR: [u8; 32] = [2u8; 32];

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_1_REDELEGATE_AMOUNT: u64 = 32_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2: a bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::ed25519_from(ACCOUNT_2_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
    ];

    // redelegate request from ACCOUNT_2 to self.
    let redelegate_request = ExecuteRequestBuilder::standard(
        PublicKey::ed25519_from(ACCOUNT_1_ADDR),
        CONTRACT_POS_DELEGATION,
        (
            String::from(REDELEGATE_METHOD),
            PublicKey::ed25519_from(ACCOUNT_2_ADDR),
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            U512::from(ACCOUNT_1_REDELEGATE_AMOUNT),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let _ = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(redelegate_request)
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    // assert that the delegations are not changed
    let delegation_1 = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR),
        base16::encode_lower(&ACCOUNT_1_ADDR),
        GENESIS_VALIDATOR_STAKE
    );
    let delegation_2 = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_2_ADDR),
        base16::encode_lower(&ACCOUNT_2_ADDR),
        GENESIS_VALIDATOR_STAKE
    );
    let pop_contract = builder.get_pos_contract();
    assert!(pop_contract.named_keys().contains_key(&delegation_1));
    assert!(pop_contract.named_keys().contains_key(&delegation_2));
}

#[ignore]
#[test]
fn should_fail_to_self_redelegate() {
    const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];
    const ACCOUNT_2_ADDR: [u8; 32] = [2u8; 32];

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_1_DELEGATE_AMOUNT: u64 = 32_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2: a bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::ed25519_from(ACCOUNT_2_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
    ];

    let bond_request = ExecuteRequestBuilder::standard(
        PublicKey::ed25519_from(ACCOUNT_2_ADDR),
        CONTRACT_POS_DELEGATION,
        (
            String::from(BOND_METHOD),
            U512::from(ACCOUNT_1_DELEGATE_AMOUNT),
        ),
    )
    .build();

    // delegate request from ACCOUNT_2 to ACCOUNT_1.
    let delegate_request = ExecuteRequestBuilder::standard(
        PublicKey::ed25519_from(ACCOUNT_2_ADDR),
        CONTRACT_POS_DELEGATION,
        (
            String::from(DELEGATE_METHOD),
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            U512::from(ACCOUNT_1_DELEGATE_AMOUNT),
        ),
    )
    .build();

    // redelegate request from ACCOUNT_2 which redelegates from ACCOUNT_1 to ACCOUNT_1.
    let redelegate_request = ExecuteRequestBuilder::standard(
        PublicKey::ed25519_from(ACCOUNT_2_ADDR),
        CONTRACT_POS_DELEGATION,
        (
            String::from(REDELEGATE_METHOD),
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            Some(U512::from(ACCOUNT_1_DELEGATE_AMOUNT)),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(bond_request)
        .expect_success()
        .commit()
        .exec(delegate_request)
        .expect_success()
        .commit()
        .exec(redelegate_request)
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let response = result
        .builder()
        .get_exec_response(2)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);

    // pos::Error::SelfRedelegation => 28
    assert!(error_message.contains(&format!(
        "Revert({})",
        u32::from(ApiError::ProofOfStake(28))
    )));
}

#[ignore]
#[test]
fn should_fail_to_redelegate_more_than_own_shares() {
    const ACCOUNT_1_ADDR: [u8; 32] = [1u8; 32];
    const ACCOUNT_2_ADDR: [u8; 32] = [2u8; 32];
    const ACCOUNT_3_ADDR: [u8; 32] = [3u8; 32];

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_3_DELEGATE_AMOUNT: u64 = 32_000;
    const ACCOUNT_3_REDELEGATE_AMOUNT: u64 = 42_000;

    // ACCOUNT_1: a bonded account with the initial balance.
    // ACCOUNT_2: a bonded account with the initial balance.
    // ACCOUNT_3: a not bonded account with the initial balance.
    let accounts = vec![
        GenesisAccount::new(
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::ed25519_from(ACCOUNT_2_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::ed25519_from(ACCOUNT_3_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    // bond request from ACCOUNT_3
    let bond_request = ExecuteRequestBuilder::standard(
        PublicKey::ed25519_from(ACCOUNT_3_ADDR),
        CONTRACT_POS_DELEGATION,
        (
            String::from(BOND_METHOD),
            U512::from(ACCOUNT_3_DELEGATE_AMOUNT),
        ),
    )
    .build();

    // delegate request from ACCOUNT_3 to ACCOUNT_1.
    let delegate_request = ExecuteRequestBuilder::standard(
        PublicKey::ed25519_from(ACCOUNT_3_ADDR),
        CONTRACT_POS_DELEGATION,
        (
            String::from(DELEGATE_METHOD),
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            U512::from(ACCOUNT_3_DELEGATE_AMOUNT),
        ),
    )
    .build();

    // redelegate request from ACCOUNT_3 which redelegates from ACCOUNT_1 to ACCOUNT_2.
    let redelegate_request = ExecuteRequestBuilder::standard(
        PublicKey::ed25519_from(ACCOUNT_3_ADDR),
        CONTRACT_POS_DELEGATION,
        (
            String::from(REDELEGATE_METHOD),
            PublicKey::ed25519_from(ACCOUNT_1_ADDR),
            PublicKey::ed25519_from(ACCOUNT_2_ADDR),
            Some(U512::from(ACCOUNT_3_REDELEGATE_AMOUNT)),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(bond_request)
        .expect_success()
        .commit()
        .exec(delegate_request)
        .expect_success()
        .commit()
        .exec(redelegate_request)
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let response = result
        .builder()
        .get_exec_response(2)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);

    // pos::Error::UndelegateTooLarge => 30
    assert!(error_message.contains(&format!(
        "Revert({})",
        u32::from(ApiError::ProofOfStake(30))
    )));
}
