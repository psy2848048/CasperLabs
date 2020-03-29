use num_traits::identities::Zero;

use engine_core::engine_state::{genesis::GenesisAccount, CONV_RATE};
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder},
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{account::PublicKey, U512};

const CONTRACT_POS_VOTE: &str = "pos_delegation.wasm";

const METHOD_WRITE_GENESIS_TOTAL_SUPPLY: &str = "write_genesis_total_supply";
const METHOD_DISTRIBUTE: &str = "distribute";
const METHOD_CLAIM_COMMISSION: &str = "claim_commission";
const METHOD_CLAIM_REWARD: &str = "claim_reward";
const METHOD_DELEGATE: &str = "delegate";

#[ignore]
#[test]
fn should_run_successful_distribute() {
    const SYSTEM_ADDR: [u8; 32] = [0u8; 32];
    const ACCOUNT_1_ADDR_DAPP_1: [u8; 32] = [1u8; 32];
    const ACCOUNT_2_ADDR_DAPP_2: [u8; 32] = [2u8; 32];
    const ACCOUNT_3_ADDR_USER_1: [u8; 32] = [3u8; 32];
    const ACCOUNT_4_ADDR_USER_2: [u8; 32] = [4u8; 32];
    const ACCOUNT_5_ADDR_USER_3: [u8; 32] = [5u8; 32];

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_3_DELEGATE_AMOUNT: u64 = 10_000;

    let accounts = vec![
        GenesisAccount::new(
            PublicKey::new(SYSTEM_ADDR),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
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
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
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
    assert!(pos_contract
        .named_keys()
        .contains_key(&lookup_key_delegation));

    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_3_ADDR_USER_1),
        GENESIS_VALIDATOR_STAKE
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    println!("Here we are");
    println!("1. write genesis supply");

    let write_genesis_supply_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR_DAPP_1,
        CONTRACT_POS_VOTE,
        (
            String::from(METHOD_WRITE_GENESIS_TOTAL_SUPPLY),
            U512::from(2_000_000_000) * U512::from(CONV_RATE),
        ),
    )
    .build();

    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(write_genesis_supply_request)
        .expect_success()
        .commit()
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("t_")
            })
            .count(),
        1
    );

    // setup done. start distribute

    println!("2. distribute");

    let distribution_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR_DAPP_1,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_DISTRIBUTE),),
    )
    .build();

    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(distribution_request)
        .expect_success()
        .commit()
        .finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be a still only one validator.
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("c_")
            })
            .count(),
        3
    );
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("r_"))
            .count(),
        3
    );

    // Delegate some amount and try distribute

    let delegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR_DAPP_2,
        CONTRACT_POS_VOTE,
        (
            String::from(METHOD_DELEGATE),
            PublicKey::new(ACCOUNT_2_ADDR_DAPP_2),
            U512::from(ACCOUNT_3_DELEGATE_AMOUNT),
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

    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("c_")
            })
            .count(),
        3
    );

    println!("Delegation done");

    let distribution_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR_DAPP_1,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_DISTRIBUTE),),
    )
    .build();

    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(distribution_request)
        .expect_success()
        .commit()
        .finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("c_")
            })
            .count(),
        4
    );

    println!("3. Claim");

    let claim_commission_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR_DAPP_1,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_CLAIM_COMMISSION),),
    )
    .build();

    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(claim_commission_request)
        .expect_success()
        .commit()
        .finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("c_")
            })
            .count(),
        3
    );

    println!("4. Reward");

    let reward_commission_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR_DAPP_1,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_CLAIM_REWARD),),
    )
    .build();

    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _result = builder
        .exec(reward_commission_request)
        .expect_success()
        .commit()
        .finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("r_")
            })
            .count(),
        3
    );
}
