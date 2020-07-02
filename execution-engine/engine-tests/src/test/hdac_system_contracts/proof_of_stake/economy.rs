use lazy_static::lazy_static;

use std::convert::TryFrom;

use engine_core::engine_state::genesis::GenesisAccount;
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, StepRequestBuilder},
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{account::PublicKey, bytesrepr::ToBytes, CLValue, Key, U512};

const CONTRACT_POS_VOTE: &str = "pos_delegation.wasm";

const METHOD_CLAIM_COMMISSION: &str = "claim_commission";
const METHOD_CLAIM_REWARD: &str = "claim_reward";
const METHOD_DELEGATE: &str = "delegate";
const METHOD_BOND: &str = "bond";

const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;
lazy_static! {
    static ref GENESIS_TOTAL_SUPPLY: U512 = U512::from(2_000_000_020) * BIGSUN_TO_HDAC;
    static ref GENESIS_VALIDATOR_STAKE: U512 = U512::from(1_000_000_000) * BIGSUN_TO_HDAC;
}

fn query_commission_amount(builder: &InMemoryWasmTestBuilder, validator: &PublicKey) -> U512 {
    let pop_uref = builder.get_pos_contract_uref();
    let key = {
        let mut ret = Vec::with_capacity(1 + validator.as_bytes().len());
        ret.push(32u8);
        ret.extend(validator.as_bytes());
        Key::local(pop_uref.addr(), &ret.to_bytes().unwrap())
    };
    let got: CLValue = builder
        .query(None, key.clone(), &[])
        .and_then(|v| CLValue::try_from(v).map_err(|error| format!("{:?}", error)))
        .expect("should have local value.");
    let got: U512 = got.into_t().unwrap();
    got
}

fn query_reward_amount(builder: &InMemoryWasmTestBuilder, delegator: &PublicKey) -> U512 {
    let pop_uref = builder.get_pos_contract_uref();
    let key = {
        let mut ret = Vec::with_capacity(1 + delegator.as_bytes().len());
        ret.push(33u8);
        ret.extend(delegator.as_bytes());
        Key::local(pop_uref.addr(), &ret.to_bytes().unwrap())
    };
    let got: CLValue = builder
        .query(None, key.clone(), &[])
        .and_then(|v| CLValue::try_from(v).map_err(|error| format!("{:?}", error)))
        .expect("should have local value.");
    let got: U512 = got.into_t().unwrap();
    got
}

#[ignore]
#[test]
fn should_run_successful_step() {
    const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR: PublicKey = PublicKey::ed25519_from([2u8; 32]);

    const ACCOUNT_2_DELEGATE_AMOUNT: u64 = BIGSUN_TO_HDAC;

    // ACCOUNT_1 and ACCOUNT_2 bond and self-delegate.
    let accounts = vec![
        GenesisAccount::new(
            ACCOUNT_1_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(*GENESIS_VALIDATOR_STAKE),
        ),
        GenesisAccount::new(
            ACCOUNT_2_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(*GENESIS_VALIDATOR_STAKE),
        ),
    ];

    // ACCOUNT_2 bond additionally
    let bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_VOTE,
        (
            String::from(METHOD_BOND),
            U512::from(BIGSUN_TO_HDAC),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(bond_request)
        .expect_success()
        .commit()
        .finish();

    // #1 assert total_supply
    {
        let pop_uref = builder.get_pos_contract_uref();
        let key = Key::local(pop_uref.addr(), &[0u8; 1]);
        let got: CLValue = builder
            .query(None, key.clone(), &[])
            .and_then(|v| CLValue::try_from(v).map_err(|error| format!("{:?}", error)))
            .expect("should have local value.");
        let got: U512 = got.into_t().unwrap();
        assert_eq!(got, *GENESIS_TOTAL_SUPPLY);
    }

    // #2 distribute
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder.step(StepRequestBuilder::default().build()).finish();

    // #3 assert commission and reward table
    assert!(query_commission_amount(&builder, &ACCOUNT_1_ADDR) > U512::zero()); // ACCOUNT_1's commission
    assert!(query_commission_amount(&builder, &ACCOUNT_2_ADDR) > U512::zero()); // ACCOUNT_2's commission
    assert!(query_reward_amount(&builder, &ACCOUNT_1_ADDR) > U512::zero()); // ACCOUNT_1's reward
    assert!(query_reward_amount(&builder, &ACCOUNT_2_ADDR) > U512::zero()); // ACCOUNT_2's reward

    // #4-1 ACCOUNT_2 delegates to ACCOUNT_1
    // #4-2 Arouse commission distribution through step
    // #4-3 ACCOUNT_1 claims commission
    let delegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_VOTE,
        (
            String::from(METHOD_DELEGATE),
            ACCOUNT_1_ADDR,
            U512::from(ACCOUNT_2_DELEGATE_AMOUNT),
        ),
    )
    .build();

    let claim_commission_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_CLAIM_COMMISSION),),
    )
    .build();

    // get ACCOUNT_1's balance before commission transfer.
    let account_1 = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("account should exist");
    let account_1_balance_before = builder.get_purse_balance(account_1.main_purse());

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(delegate_request) // #4-1 ACCOUNT_2 to ACCOUNT_1
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build()) // #4-2 distribute
        .exec(claim_commission_request) // #4-3 ACCOUNT_1 claims commission
        .expect_success()
        .commit()
        .finish();

    // #5 assert ACCOUNT_1's commission is withdrawed.
    assert!(query_commission_amount(&builder, &ACCOUNT_1_ADDR) == U512::zero());

    // #6 assert commission claim effect
    // get ACCOUNT_1's balance after commission transfer.
    let account_1_balance_after = builder.get_purse_balance(account_1.main_purse());

    assert!(account_1_balance_before < account_1_balance_after);

    // #7 ACCOUNT_1 claims reward.
    let claim_reward_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_CLAIM_REWARD),),
    )
    .build();

    // get ACCOUNT_1's balance before reward transfer.
    let account_1 = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("account should exist");
    let account_1_balance_before = builder.get_purse_balance(account_1.main_purse());

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(claim_reward_request)
        .expect_success()
        .commit()
        .finish();

    // #8 assert ACCOUNT_1's reward is withdrawed.
    assert!(query_reward_amount(&builder, &ACCOUNT_1_ADDR) == U512::zero());

    // #9 assert reward claim effect
    // get ACCOUNT_1's balance after reward transfer.
    let account_1_balance_after = builder.get_purse_balance(account_1.main_purse());

    assert!(account_1_balance_before < account_1_balance_after);
}
