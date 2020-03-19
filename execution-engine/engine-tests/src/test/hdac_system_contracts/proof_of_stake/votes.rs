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
    ApiError, Key, U512,
};

const CONTRACT_POS_VOTE: &str = "pos_delegation.wasm";

const BOND_METHOD: &str = "bond";
const UNBOND_METHOD: &str = "unbond";
const VOTE_METHOD: &str = "vote";
const UNVOTE_METHOD: &str = "unvote";

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
fn should_run_successful_vote_and_unvote_after_bonding() {
    const ACCOUNT_1_ADDR_DAPP_1: [u8; 32] = [1u8; 32];
    const ACCOUNT_2_ADDR_DAPP_2: [u8; 32] = [2u8; 32];
    const ACCOUNT_3_ADDR_USER_1: [u8; 32] = [3u8; 32];
    const ACCOUNT_4_ADDR_USER_2: [u8; 32] = [4u8; 32];
    const ACCOUNT_5_ADDR_USER_3: [u8; 32] = [5u8; 32];

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_3_VOTE_AMOUNT: u64 = 10_000;
    const ACCOUNT_3_UNVOTE_AMOUNT: u64 = 5_000;

    let accounts = vec![
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_1_ADDR_DAPP_1),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_2_ADDR_DAPP_2),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_3_ADDR_USER_1),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_4_ADDR_USER_2),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_5_ADDR_USER_3),
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
    let lookup_key_delegation = format!(
        "d_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR_DAPP_1),
        base16::encode_lower(&ACCOUNT_1_ADDR_DAPP_1),
        GENESIS_VALIDATOR_STAKE
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key_delegation));

    println!("1-1 finished");

    // setup done. start testing
    // execute vote
    // vote by ACCOUNT_3_ADDR_USER_1 to ACCOUNT_1_ADDR_DAPP_1

    let vote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR_USER_1,
        CONTRACT_POS_VOTE,
        (
            String::from(VOTE_METHOD),
            PublicKey::new(ACCOUNT_3_ADDR_USER_1),
            PublicKey::new(ACCOUNT_1_ADDR_DAPP_1),
            U512::from(ACCOUNT_3_VOTE_AMOUNT),
        ),
    )
    .build();
    println!("1-2 build finished");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(vote_request)
        .expect_success()
        .commit()
        .finish();

    println!("1-2 execution finished");

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
    println!("1-2-1 finished");

    // that validator should be a_{dApp_pubkey}_{user_pubkey}_{voted_amount}
    let lookup_key_vote = format!(
        "a_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_3_ADDR_USER_1),
        base16::encode_lower(&ACCOUNT_1_ADDR_DAPP_1),
        ACCOUNT_3_VOTE_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key_vote));

    // there should be 1 vote
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("a_"))
            .count(),
        1
    );

    println!("1-2 finished");

    // execute second vote with user 1 to another dapp
    let vote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR_USER_1,
        CONTRACT_POS_VOTE,
        (
            String::from(VOTE_METHOD),
            PublicKey::new(ACCOUNT_3_ADDR_USER_1),
            PublicKey::new(ACCOUNT_2_ADDR_DAPP_2),
            U512::from(ACCOUNT_3_VOTE_AMOUNT),
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(vote_request)
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
            .filter(|(key, _)| key.starts_with("a_"))
            .count(),
        2
    );

    // execute unvote
    // unvote {ACCOUNT_2}_{ACCOUNT_1}_{ACCOUNT_2_UNDELEGATE_AMOUNT}
    let unvote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR_USER_1,
        CONTRACT_POS_VOTE,
        (
            String::from(UNVOTE_METHOD),
            PublicKey::new(ACCOUNT_3_ADDR_USER_1),
            PublicKey::new(ACCOUNT_1_ADDR_DAPP_1),
            None::<U512>,
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(unvote_request)
        .expect_success()
        .commit()
        .finish();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // validate validator stake amount
    let lookup_key_vote = format!(
        "a_{}_{}_{}",
        base16::encode_lower(&ACCOUNT_3_ADDR_USER_1),
        base16::encode_lower(&ACCOUNT_1_ADDR_DAPP_1),
        ACCOUNT_3_VOTE_AMOUNT
    );
    assert!(!pos_contract.named_keys().contains_key(&lookup_key_vote));

    // there should be still 2 delegations
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("a_"))
            .count(),
        1
    );
    println!("1-3 finished");
}

#[ignore]
#[test]
fn should_fail_to_vote_more_than_bonded() {
    // 1. Try to vote twice.
    // 2. Second vote, the amount of vote exceeds than user's bond, an error expected
    const ACCOUNT_1_ADDR_DAPP_1: [u8; 32] = [1u8; 32];
    const ACCOUNT_2_ADDR_DAPP_2: [u8; 32] = [2u8; 32];
    const ACCOUNT_3_ADDR_USER_1: [u8; 32] = [3u8; 32];
    
    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_3_VOTE_AMOUNT: u64 = 30_000;

    let accounts = vec![
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_1_ADDR_DAPP_1),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_2_ADDR_DAPP_2),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
        GenesisAccount::new(
            PublicKey::new(ACCOUNT_3_ADDR_USER_1),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];
    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts))
        .finish();

    //
    // first vote. working well expected
    //
    let vote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR_USER_1,
        CONTRACT_POS_VOTE,
        (
            String::from(VOTE_METHOD),
            PublicKey::new(ACCOUNT_1_ADDR_DAPP_1),
            PublicKey::new(ACCOUNT_3_ADDR_USER_1),
            U512::from(ACCOUNT_3_VOTE_AMOUNT),
        ),
    )
    .build();

    let result = builder
        .exec(vote_request)
        .expect_success()
        .commit()
        .finish();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let pos_uref = builder.get_pos_contract_uref();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be a still only one validator.
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("a_"))
            .count(),
        1
    );
    println!("2-1 finished");

    //
    // second vote. an error expected
    //

    let vote_request = ExecuteRequestBuilder::standard(
        ACCOUNT_3_ADDR_USER_1,
        CONTRACT_POS_VOTE,
        (
            String::from(VOTE_METHOD),
            PublicKey::new(ACCOUNT_1_ADDR_DAPP_1),
            PublicKey::new(ACCOUNT_3_ADDR_USER_1),
            U512::from(ACCOUNT_3_VOTE_AMOUNT),
        ),
    )
    .build();



    let result = builder
        .exec(vote_request)
        .expect_success()
        .commit()
        .finish();

    let response = result
        .builder()
        .get_exec_response(0)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);

    // pos::Error::NotBonded => 0
    assert!(error_message.contains(&format!("Revert({})", u32::from(ApiError::ProofOfStake(0)))));
    println!("2-2 finished");
}
