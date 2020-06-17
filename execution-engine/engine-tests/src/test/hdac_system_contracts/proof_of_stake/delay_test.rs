use std::convert::TryFrom;

use engine_core::engine_state::genesis::POS_BONDING_PURSE;
use engine_test_support::{
    internal::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, StepRequestBuilder, DEFAULT_GENESIS_CONFIG,
    },
    DEFAULT_ACCOUNT_ADDR,
};
use types::{account::PublicKey, bytesrepr::ToBytes, BlockTime, CLValue, Key, URef, U512};

const CONTRACT_POS_DELEGATION: &str = "pos_delegation.wasm";
const METHOD_BOND: &str = "bond";
const METHOD_UNBOND: &str = "unbond";

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

fn assert_bond_amount(builder: &InMemoryWasmTestBuilder, address: &PublicKey, amount: U512) {
    let pop_uref = builder.get_pos_contract_uref();
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

// 2 days in seconds
const UNBONDING_DELAY: u64 = 2 * 24 * 60 * 60;

#[test]
#[ignore]
fn should_unbond_only_the_mature_requests() {
    const BOND_AMOUNT: u64 = 50_000;
    const UNBOND_AMOUNT_1: u64 = 12_000;
    const UNBOND_AMOUNT_2: u64 = 22_000;
    const UNBOND_REQUEST_TIMESTAMP_1: u64 = 10;
    const UNBOND_REQUEST_TIMESTAMP_2: u64 = UNBOND_REQUEST_TIMESTAMP_1 + 1;

    // #1 bond 50k, timestamp: 0
    // #2 unbond 12k, timestamp: 10
    // #3 unbond 23k, timestamp: 20
    // #4 step timestamp: UNBONDING_DELAY + 10
    // #5 assert_bond_amount(38k)
    let bond_request = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_DELEGATION,
        (String::from(METHOD_BOND), U512::from(BOND_AMOUNT)),
    )
    .build();
    let mut unbond_request_1 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(METHOD_UNBOND),
            Some(U512::from(UNBOND_AMOUNT_1)),
        ),
    )
    .build();
    unbond_request_1.block_time = UNBOND_REQUEST_TIMESTAMP_1;
    let mut unbond_request_2 = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_DELEGATION,
        (
            String::from(METHOD_UNBOND),
            Some(U512::from(UNBOND_AMOUNT_2)),
        ),
    )
    .build();
    unbond_request_2.block_time = UNBOND_REQUEST_TIMESTAMP_2;

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&*DEFAULT_GENESIS_CONFIG)
        .exec(bond_request)
        .expect_success()
        .commit()
        .exec(unbond_request_1)
        .expect_success()
        .commit()
        .exec(unbond_request_2)
        .expect_success()
        .commit()
        .finish();

    // Unbond is processed in the step but the step is currently not supporting to propagate the
    // errors. Therefore, assert by checking that the states amount are not changed.

    let default_account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should get default_account");
    let balance_before_step = builder.get_purse_balance(default_account.main_purse());

    let step_request = StepRequestBuilder::default()
        .with_blocktime(BlockTime::new(UNBONDING_DELAY + UNBOND_REQUEST_TIMESTAMP_1))
        .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _ = builder
        .step(step_request) // deliver only unbond_request_1
        .finish();

    let balance_after_step = builder.get_purse_balance(default_account.main_purse());

    // check default_account's balance
    assert_eq!(balance_before_step + UNBOND_AMOUNT_1, balance_after_step);

    // check bond amount
    assert_bond_amount(
        &builder,
        &DEFAULT_ACCOUNT_ADDR,
        U512::from(BOND_AMOUNT - UNBOND_AMOUNT_1),
    );
    // check the balance of bonding_purse
    assert_eq!(
        U512::from(BOND_AMOUNT - UNBOND_AMOUNT_1),
        get_pos_bonding_purse_balance(&builder)
    );
}
