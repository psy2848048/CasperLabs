use num_traits::identities::Zero;
use std::convert::TryFrom;

use engine_core::engine_state::genesis::GenesisAccount;
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_GENESIS_CONFIG},
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{account::PublicKey, bytesrepr::ToBytes, ApiError, CLValue, Key, URef, U512};

const CONTRACT_POS_VOTE: &str = "pos_delegation.wasm";

const BOND_METHOD: &str = "bond";
const VOTE_METHOD: &str = "vote";
const UNVOTE_METHOD: &str = "unvote";

fn assert_vote_amount(
    pop_uref: &URef,
    voter: &PublicKey,
    dapp: &Key,
    amount: U512,
    builder: &InMemoryWasmTestBuilder,
) {
    let key = {
        let mut ret = Vec::with_capacity(1 + voter.as_bytes().len() + dapp.serialized_length());
        ret.push(2u8); // voting prefix
        ret.extend(voter.as_bytes());
        ret.extend(dapp.to_bytes().expect("Key to bytes failed").into_iter());
        Key::local(pop_uref.addr(), &ret.to_bytes().unwrap())
    };
    let got: CLValue = builder
        .query(None, key.clone(), &[])
        .and_then(|v| CLValue::try_from(v).map_err(|error| format!("{:?}", error)))
        .expect("should have local value.");
    let got: U512 = got.into_t().unwrap();
    assert_eq!(got, amount, "vote amount assertion failure for {:?}", voter);
}

fn assert_voting_amount(
    pop_uref: &URef,
    voter: &PublicKey,
    amount: U512,
    builder: &InMemoryWasmTestBuilder,
) {
    let key = {
        let mut ret = Vec::with_capacity(1 + voter.as_bytes().len());
        ret.push(2u8); // voting prefix
        ret.extend(voter.as_bytes());
        Key::local(pop_uref.addr(), &ret.to_bytes().unwrap())
    };
    let got: CLValue = builder
        .query(None, key.clone(), &[])
        .and_then(|v| CLValue::try_from(v).map_err(|error| format!("{:?}", error)))
        .expect("should have local value.");
    let got: U512 = got.into_t().unwrap();
    assert_eq!(
        got, amount,
        "voting amount assertion failure for {:?}",
        voter
    );
}

fn assert_voted_amount(
    pop_uref: &URef,
    dapp: &Key,
    amount: U512,
    builder: &InMemoryWasmTestBuilder,
) {
    let key = {
        let mut ret = Vec::with_capacity(1 + dapp.serialized_length());
        ret.push(3u8); // vote prefix
        ret.extend(dapp.to_bytes().expect("Key to bytes failed").into_iter());
        Key::local(pop_uref.addr(), &ret.to_bytes().unwrap())
    };
    let got: CLValue = builder
        .query(None, key.clone(), &[])
        .and_then(|v| CLValue::try_from(v).map_err(|error| format!("{:?}", error)))
        .expect("should have local value.");
    let got: U512 = got.into_t().unwrap();
    assert_eq!(got, amount, "voted amount assertion failure for {:?}", dapp);
}

#[ignore]
#[test]
fn should_run_successful_vote_and_unvote_after_bonding() {
    const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR: PublicKey = PublicKey::ed25519_from([2u8; 32]);
    const DAPP_1_ADDR: Key = Key::Hash([11u8; 32]);
    const ACCOUNT_1_VOTE_AMOUNT: u64 = 10_000;
    const ACCOUNT_1_UNVOTE_AMOUNT: u64 = 4_800;
    const ACCOUNT_2_VOTE_AMOUNT: u64 = 20_000;

    let accounts = vec![
        GenesisAccount::new(
            ACCOUNT_1_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
        GenesisAccount::new(
            ACCOUNT_2_ADDR,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    let bond_1_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(BOND_METHOD), U512::from(ACCOUNT_1_VOTE_AMOUNT)),
    )
    .build();
    let bond_2_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(BOND_METHOD), U512::from(ACCOUNT_2_VOTE_AMOUNT)),
    )
    .build();

    // #1 account_1 votes to dapp_1
    let vote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_VOTE,
        (
            String::from(VOTE_METHOD),
            DAPP_1_ADDR,
            U512::from(ACCOUNT_1_VOTE_AMOUNT),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, Default::default()))
        .exec(bond_1_request)
        .expect_success()
        .commit()
        .exec(bond_2_request)
        .expect_success()
        .commit()
        .exec(vote_request)
        .expect_success()
        .commit()
        .finish();

    let pop_uref = builder.get_pos_contract_uref();

    // #2 assert vote {account_1, dapp1, amount}
    assert_vote_amount(
        &pop_uref,
        &ACCOUNT_1_ADDR,
        &DAPP_1_ADDR,
        ACCOUNT_1_VOTE_AMOUNT.into(),
        &builder,
    );
    assert_voting_amount(
        &pop_uref,
        &ACCOUNT_1_ADDR,
        ACCOUNT_1_VOTE_AMOUNT.into(),
        &builder,
    );
    assert_voted_amount(
        &pop_uref,
        &DAPP_1_ADDR,
        ACCOUNT_1_VOTE_AMOUNT.into(),
        &builder,
    );

    // #3 ACCOUNT_1 unvotes to DAPP_1
    let unvote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_VOTE,
        (
            String::from(UNVOTE_METHOD),
            DAPP_1_ADDR,
            Some(U512::from(ACCOUNT_1_UNVOTE_AMOUNT)),
        ),
    )
    .build();
    // #4 ACCOUNT_2 votes DAPP_1
    let vote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_VOTE,
        (
            String::from(VOTE_METHOD),
            DAPP_1_ADDR,
            U512::from(ACCOUNT_2_VOTE_AMOUNT),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(unvote_request)
        .expect_success()
        .commit()
        .exec(vote_request)
        .expect_success()
        .commit()
        .finish();

    // #5 assert vote {ACCOUNT_1, DAPP_1, amount} after unvote
    assert_vote_amount(
        &pop_uref,
        &ACCOUNT_1_ADDR,
        &DAPP_1_ADDR,
        (ACCOUNT_1_VOTE_AMOUNT - ACCOUNT_1_UNVOTE_AMOUNT).into(),
        &builder,
    );
    // #6 assert voting amount of ACCOUNT_1 after unvote
    assert_voting_amount(
        &pop_uref,
        &ACCOUNT_1_ADDR,
        (ACCOUNT_1_VOTE_AMOUNT - ACCOUNT_1_UNVOTE_AMOUNT).into(),
        &builder,
    );
    // #7 assert voted amount of DAPP_1 after unvote and vote of ACCOUNT_2
    assert_voted_amount(
        &pop_uref,
        &DAPP_1_ADDR,
        (ACCOUNT_1_VOTE_AMOUNT - ACCOUNT_1_UNVOTE_AMOUNT + ACCOUNT_2_VOTE_AMOUNT).into(),
        &builder,
    );

    // #8 unvote all
    let unvote_1_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(UNVOTE_METHOD), DAPP_1_ADDR, None::<U512>),
    )
    .build();
    let unvote_2_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(UNVOTE_METHOD), DAPP_1_ADDR, None::<U512>),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _ = builder
        .exec(unvote_1_request)
        .expect_success()
        .commit()
        .exec(unvote_2_request)
        .expect_success()
        .commit()
        .finish();

    // #8 assert voted amount of DAPP_1
    assert_voted_amount(&pop_uref, &DAPP_1_ADDR, U512::zero(), &builder);
    assert_voting_amount(&pop_uref, &ACCOUNT_1_ADDR, U512::zero(), &builder);
    assert_voting_amount(&pop_uref, &ACCOUNT_2_ADDR, U512::zero(), &builder);
}

#[ignore]
#[test]
fn should_fail_to_vote_more_than_bonded() {
    let bond_amount = U512::from(1000);
    let vote_amount = U512::from(1001);
    let dapp_addr = Key::Hash([11u8; 32]);

    let bond_request = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(BOND_METHOD), bond_amount),
    )
    .build();
    let vote_request = ExecuteRequestBuilder::standard(
        DEFAULT_ACCOUNT_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(VOTE_METHOD), dapp_addr, vote_amount),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&DEFAULT_GENESIS_CONFIG)
        .exec(bond_request)
        .expect_success()
        .commit()
        .exec(vote_request)
        .commit()
        .finish();

    let response = result
        .builder()
        .get_exec_response(1)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);

    assert!(error_message.contains(&format!(
        "Revert({})",
        u32::from(ApiError::ProofOfStake(39))
    )));
}
