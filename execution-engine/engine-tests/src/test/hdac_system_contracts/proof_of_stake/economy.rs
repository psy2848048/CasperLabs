use lazy_static::lazy_static;

use engine_core::engine_state::genesis::GenesisAccount;
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, StepRequestBuilder},
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{account::PublicKey, U512};

const CONTRACT_POS_VOTE: &str = "pos_delegation.wasm";

const METHOD_CLAIM_COMMISSION: &str = "claim_commission";
const METHOD_CLAIM_REWARD: &str = "claim_reward";
const METHOD_DELEGATE: &str = "delegate";
const METHOD_BOND: &str = "bond";

const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;
lazy_static! {
    static ref GENESIS_TOTAL_SUPPLY: U512 = U512::from(2_000_000_000) * BIGSUN_TO_HDAC;
}

#[ignore]
#[test]
fn should_run_successful_step() {
    const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR: PublicKey = PublicKey::ed25519_from([2u8; 32]);

    const GENESIS_VALIDATOR_STAKE: u64 = 5u64 * BIGSUN_TO_HDAC;
    const ACCOUNT_2_DELEGATE_AMOUNT: u64 = BIGSUN_TO_HDAC;

    // Genesis accounts bond and self-delegate.
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
    ];

    // ACCOUNT_2 bond additionally
    let bond_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR,
        CONTRACT_POS_VOTE,
        (
            String::from(METHOD_BOND),
            U512::from(GENESIS_VALIDATOR_STAKE),
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
    let pos_contract = builder.get_pos_contract();
    assert!(pos_contract
        .named_keys()
        .contains_key(&format!("t_{}", *GENESIS_TOTAL_SUPPLY)));

    // #2 distribute
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder.step(StepRequestBuilder::default().build()).finish();

    // #3 assert commission and reward entries
    // get updated contract context
    let pos_contract = builder.get_pos_contract();
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| { key.starts_with("c_") })
            .count(),
        2
    );
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("r_"))
            .count(),
        2
    );

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

    // #5 assert commission states are diminished.
    // get updated contract context
    let pos_contract = builder.get_pos_contract();
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| { key.starts_with("c_") })
            .count(),
        1
    );

    // #6 assert commission claim effect
    // get ACCOUNT_1's balance before commission transfer.
    let account_1 = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("account should exist");
    let account_1_balance_before = builder.get_purse_balance(account_1.main_purse());

    // System transfer commission amount to ACCOUNT_1
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder.step(StepRequestBuilder::default().build()).finish();

    // get ACCOUNT_1's balance after commission transfer.
    let account_1_balance_after = builder.get_purse_balance(account_1.main_purse());

    assert!(account_1_balance_before < account_1_balance_after);

    // #7 ACCOUNT_1 claims reward.
    let reward_commission_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_CLAIM_REWARD),),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(reward_commission_request)
        .expect_success()
        .commit()
        .finish();

    // #8 assert reward states are diminished.
    // get updated contract context
    let pos_contract = builder.get_pos_contract();
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| { key.starts_with("r_") })
            .count(),
        1
    );

    // #9 assert commission claim effect
    // get ACCOUNT_1's balance before reward transfer.
    let account_1 = builder
        .get_account(ACCOUNT_1_ADDR)
        .expect("account should exist");
    let account_1_balance_before = builder.get_purse_balance(account_1.main_purse());

    // System transfer reward amount to ACCOUNT_1
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _ = builder.step(StepRequestBuilder::default().build()).finish();

    // get ACCOUNT_1's balance after reward transfer.
    let account_1_balance_after = builder.get_purse_balance(account_1.main_purse());

    assert!(account_1_balance_before < account_1_balance_after);
}
