use std::convert::TryFrom;

use engine_core::engine_state::genesis::{GenesisAccount, POS_BONDING_PURSE};
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{
        utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, StepRequestBuilder,
        DEFAULT_ACCOUNTS, DEFAULT_PAYMENT,
    },
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{account::PublicKey, bytesrepr::ToBytes, ApiError, CLValue, Key, URef, U512};

const CONTRACT_POS_BONDING: &str = "pos_bonding.wasm";
const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
const ACCOUNT_1_STAKE: u64 = 42_000;
const ACCOUNT_1_UNBOND_1: u64 = 22_000;
const ACCOUNT_1_UNBOND_2: u64 = 20_000;

const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
const GENESIS_ACCOUNT_STAKE: u64 = 100_000;
const GENESIS_ACCOUNT_UNBOND_1: u64 = 45_000;
const GENESIS_ACCOUNT_UNBOND_2: u64 = 55_000;

const TEST_BOND: &str = "bond";
const TEST_BOND_FROM_MAIN_PURSE: &str = "bond-from-main-purse";
const TEST_SEED_NEW_ACCOUNT: &str = "seed_new_account";
const TEST_UNBOND: &str = "unbond";

fn get_pos_purse_id_by_name(builder: &InMemoryWasmTestBuilder, purse_name: &str) -> Option<URef> {
    let pos_contract = builder.get_pos_contract();

    pos_contract
        .named_keys()
        .get(purse_name)
        .and_then(Key::as_uref)
        .cloned()
}

fn get_pos_bonding_purse_balance(builder: &InMemoryWasmTestBuilder) -> U512 {
    let purse_id = get_pos_purse_id_by_name(builder, POS_BONDING_PURSE)
        .expect("should find PoS payment purse");
    builder.get_purse_balance(purse_id)
}

fn assert_bond_amount(
    pop_uref: &URef,
    address: &PublicKey,
    amount: U512,
    builder: &InMemoryWasmTestBuilder,
) {
    let key = {
        let mut ret = Vec::with_capacity(1 + address.as_bytes().len());
        ret.push(1u8);
        ret.extend(address.as_bytes());
        Key::local(pop_uref.addr(), &ret.to_bytes().unwrap())
    };
    let got: CLValue = builder
        .query(None, key.clone(), &[])
        .and_then(|v| CLValue::try_from(v).map_err(|error| format!("{:?}", error)))
        .expect("should have local value.");
    let got: U512 = got.into_t().unwrap();
    assert_eq!(
        got, amount,
        "bond amount assertion failure for {:?}",
        address
    );
}

#[ignore]
#[test]
fn should_run_successful_bond_and_unbond() {
    let account_1_seed_amount = *DEFAULT_PAYMENT * 10 * 2;
    // default_account:
    // {balance: DEFAULT_ACCOUNT_INITIAL_BALANCE, stake: 0}
    // genesis_validator[42; 32]:
    // { balance: 100k, self_delegation: 50k }
    let accounts = {
        let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
        let account = GenesisAccount::new(
            PublicKey::ed25519_from([42; 32]),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()) * Motes::new(2.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        );
        tmp.push(account);
        tmp
    };

    let genesis_config = utils::create_genesis_config(accounts, Default::default());

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder.run_genesis(&genesis_config).finish();

    let default_account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should get account 1");

    let pos = builder.get_pos_contract_uref();

    // #1 default_account bond 100k
    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_BONDING,
        (String::from(TEST_BOND), U512::from(GENESIS_ACCOUNT_STAKE)),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .finish();

    // #2 assert default_account's bond amount
    assert_bond_amount(
        &pos,
        &DEFAULT_ACCOUNT_ADDR,
        GENESIS_ACCOUNT_STAKE.into(),
        &builder,
    );

    // #3 assert bonding purse balance;
    // default_account(GENESIS_ACCOUNT_STAKE(100k)) +
    // genesis_validator[42;32](GENESIS_VALIDATOR_STAKE(50k))
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(GENESIS_VALIDATOR_STAKE + GENESIS_ACCOUNT_STAKE)
    );

    // #4 seed account_1 and bond ACCOUNT_1_STAKE(42k)
    let exec_request_2 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_BONDING,
        (
            String::from(TEST_SEED_NEW_ACCOUNT),
            ACCOUNT_1_ADDR,
            account_1_seed_amount,
        ),
    )
    .build();

    let exec_request_3 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_BONDING,
        (
            String::from(TEST_BOND_FROM_MAIN_PURSE),
            U512::from(ACCOUNT_1_STAKE),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(exec_request_2)
        .expect_success()
        .commit()
        .exec(exec_request_3)
        .expect_success()
        .commit()
        .finish();

    let account_1 = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("should get account 1");

    let pos = builder.get_pos_contract_uref();

    // #5 assert account_1's bond amount(42k)
    assert_bond_amount(&pos, &ACCOUNT_1_ADDR, ACCOUNT_1_STAKE.into(), &builder);

    // #6 assert bonding purse balance;
    // default_account(GENESIS_ACCOUNT_STAKE(100k)) +
    // genesis_validator[42;32](GENESIS_VALIDATOR_STAKE(50k)) +
    // account_1(ACCOUNT_1_STAKE(42k))
    let pos_bonding_purse_balance = get_pos_bonding_purse_balance(&builder);
    assert_eq!(
        pos_bonding_purse_balance,
        U512::from(GENESIS_VALIDATOR_STAKE + GENESIS_ACCOUNT_STAKE + ACCOUNT_1_STAKE)
    );

    // #7 account_1 unbond ACCOUNT_1_UNBOND_1(22k)
    let exec_request_4 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_BONDING,
        (
            String::from(TEST_UNBOND),
            Some(U512::from(ACCOUNT_1_UNBOND_1)),
        ),
    )
    .build();
    let account_1_bal_before = builder.get_purse_balance(account_1.main_purse());
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(exec_request_4)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let account_1_bal_after = builder.get_purse_balance(account_1.main_purse());

    // #8 assert account1's balance after unbond request.
    assert_eq!(
        account_1_bal_after,
        account_1_bal_before - *DEFAULT_PAYMENT + ACCOUNT_1_UNBOND_1,
    );

    // #9 assert bonding purse balance;
    // default_account(GENESIS_ACCOUNT_STAKE(100k)) +
    // genesis_validator[42;32](GENESIS_VALIDATOR_STAKE(50k)) +
    // account_1(42-22 = 20k)
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(GENESIS_VALIDATOR_STAKE + GENESIS_ACCOUNT_STAKE + ACCOUNT_1_UNBOND_2)
    );

    // #10 assert account_1's bond amount(42k-22k)
    assert_bond_amount(&pos, &ACCOUNT_1_ADDR, ACCOUNT_1_UNBOND_2.into(), &builder);

    // #11 default_account unbond 45k.
    let exec_request_5 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_BONDING,
        (
            String::from(TEST_UNBOND),
            Some(U512::from(GENESIS_ACCOUNT_UNBOND_1)),
        ),
    )
    .build();
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(exec_request_5)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    // #12 assert default_account's balance after unbond
    assert_eq!(
        builder.get_purse_balance(default_account.main_purse()),
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)
            - *DEFAULT_PAYMENT * 3
            - account_1_seed_amount
            - GENESIS_ACCOUNT_UNBOND_2,
    );

    // #13 assert bonding purse balance;
    // default_account(100k - 45k = 55k) +
    // genesis_validator[42;32](GENESIS_VALIDATOR_STAKE(50k)) +
    // account_1(42-22 = 20k)
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(GENESIS_VALIDATOR_STAKE + GENESIS_ACCOUNT_UNBOND_2 + ACCOUNT_1_UNBOND_2)
    );

    // #14 unbond all account1 with Some(TOTAL_AMOUNT(20k))
    let account_1_bal_before = builder.get_purse_balance(account_1.main_purse());

    let exec_request_6 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_BONDING,
        (
            String::from(TEST_UNBOND),
            Some(U512::from(ACCOUNT_1_UNBOND_2)),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(exec_request_6)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let account_1_bal_after = builder.get_purse_balance(account_1.main_purse());

    // #15 assert account1's balance
    assert_eq!(
        account_1_bal_after,
        account_1_bal_before - *DEFAULT_PAYMENT + ACCOUNT_1_UNBOND_2,
    );

    // #16 assert bonding purse balance;
    // default_account(100k - 45k = 55k) +
    // genesis_validator[42;32](GENESIS_VALIDATOR_STAKE(50k))
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(GENESIS_VALIDATOR_STAKE + GENESIS_ACCOUNT_UNBOND_2)
    );

    // #17 assert account_1 is not bonded anymore.
    assert_bond_amount(&pos, &ACCOUNT_1_ADDR, U512::zero(), &builder);

    // #18 unbond all default_account with None
    let exec_request_7 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_BONDING,
        (String::from(TEST_UNBOND), None as Option<U512>),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(exec_request_7)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    // #19 assert default_account's balance after unbond all
    assert_eq!(
        result
            .builder()
            .get_purse_balance(default_account.main_purse()),
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE) - *DEFAULT_PAYMENT * 4 - account_1_seed_amount
    );

    // #20 assert bonding purse balance;
    // genesis_validator[42;32](GENESIS_VALIDATOR_STAKE(50k))
    assert_eq!(
        get_pos_bonding_purse_balance(&builder),
        U512::from(GENESIS_VALIDATOR_STAKE)
    );

    // #21 assert default_account's bond amount.
    assert_bond_amount(&pos, &DEFAULT_ACCOUNT_ADDR, U512::zero(), &builder);
}

#[ignore]
#[test]
fn should_fail_bonding_with_insufficient_funds() {
    // default_account:
    // {balance: DEFAULT_ACCOUNT_INITIAL_BALANCE, stake: 0}
    // genesis_validator[42; 32]:
    // { balance: 100k, self_delegation: 50k }
    let accounts = {
        let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
        let account = GenesisAccount::new(
            PublicKey::ed25519_from([42; 32]),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()) * Motes::new(2.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        );
        tmp.push(account);
        tmp
    };

    let genesis_config = utils::create_genesis_config(accounts, Default::default());

    let exec_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_BONDING,
        (
            String::from(TEST_SEED_NEW_ACCOUNT),
            ACCOUNT_1_ADDR,
            *DEFAULT_PAYMENT + GENESIS_ACCOUNT_STAKE,
        ),
    )
    .build();
    let exec_request_2 = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_BONDING,
        (
            String::from(TEST_BOND_FROM_MAIN_PURSE),
            *DEFAULT_PAYMENT + GENESIS_ACCOUNT_STAKE,
        ),
    )
    .build();

    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&genesis_config)
        .exec(exec_request_1)
        .commit()
        .exec(exec_request_2)
        .commit()
        .finish();

    let response = result
        .builder()
        .get_exec_response(1)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);
    // pos::Error::BondTransferFailed => 8
    assert!(error_message.contains(&format!("Revert({})", u32::from(ApiError::ProofOfStake(8)))));
}

#[ignore]
#[test]
fn should_fail_unbonding_validator_without_bonding_first() {
    // default_account:
    // {balance: DEFAULT_ACCOUNT_INITIAL_BALANCE, stake: 0}
    // genesis_validator[42; 32]:
    // { balance: 100k, self_delegation: 50k }
    let accounts = {
        let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
        let account = GenesisAccount::new(
            PublicKey::ed25519_from([42; 32]),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()) * Motes::new(2.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        );
        tmp.push(account);
        tmp
    };

    let genesis_config = utils::create_genesis_config(accounts, Default::default());

    // #1 default_account try to unbond 42 without bonding.
    let exec_request = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_BONDING,
        (String::from(TEST_UNBOND), Some(U512::from(42))),
    )
    .build();

    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&genesis_config)
        .exec(exec_request)
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let response = result
        .builder()
        .get_exec_response(0)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);
    // pos::Error::UnbondTooLarge => 7
    assert!(error_message.contains(&format!("Revert({})", u32::from(ApiError::ProofOfStake(7)))));
}
