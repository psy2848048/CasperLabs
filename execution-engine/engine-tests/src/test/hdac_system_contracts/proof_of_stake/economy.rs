use num_traits::identities::Zero;

use engine_core::engine_state::genesis::GenesisAccount;
use engine_shared::motes::Motes;
use engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, StepRequestBuilder},
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use types::{account::PublicKey, U512};

const CONTRACT_POS_VOTE: &str = "pos_delegation.wasm";
const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_to_account_u512.wasm";

const METHOD_CLAIM_COMMISSION: &str = "claim_commission";
const METHOD_CLAIM_REWARD: &str = "claim_reward";
const METHOD_DELEGATE: &str = "delegate";

const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;

#[ignore]
#[test]
fn should_run_successful_step() {
    const SYSTEM_ADDR: PublicKey = PublicKey::ed25519_from([0u8; 32]);
    const ACCOUNT_1_ADDR_DAPP_1: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR_DAPP_2: PublicKey = PublicKey::ed25519_from([2u8; 32]);
    const ACCOUNT_3_ADDR_USER_1: PublicKey = PublicKey::ed25519_from([3u8; 32]);
    const ACCOUNT_4_ADDR_USER_2: PublicKey = PublicKey::ed25519_from([4u8; 32]);
    const ACCOUNT_5_ADDR_USER_3: PublicKey = PublicKey::ed25519_from([5u8; 32]);

    const GENESIS_VALIDATOR_STAKE: u64 = 5u64 * BIGSUN_TO_HDAC;
    const ACCOUNT_3_DELEGATE_AMOUNT: u64 = BIGSUN_TO_HDAC;
    const SYSTEM_ACC_SUPPORT: u64 = 5u64 * BIGSUN_TO_HDAC;

    let accounts = vec![
        // System account initiates automatically
        // Don't have to put in here
        GenesisAccount::new(
            ACCOUNT_1_ADDR_DAPP_1,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            ACCOUNT_2_ADDR_DAPP_2,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
        GenesisAccount::new(
            ACCOUNT_3_ADDR_USER_1,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::new(GENESIS_VALIDATOR_STAKE.into()),
        ),
        GenesisAccount::new(
            ACCOUNT_4_ADDR_USER_2,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
        GenesisAccount::new(
            ACCOUNT_5_ADDR_USER_3,
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            Motes::zero(),
        ),
    ];

    let state_infos = vec![
        format_args!(
            "d_{}_{}_{}",
            base16::encode_lower(&ACCOUNT_1_ADDR_DAPP_1.as_bytes()),
            base16::encode_lower(&ACCOUNT_1_ADDR_DAPP_1.as_bytes()),
            GENESIS_VALIDATOR_STAKE.to_string()
        )
        .to_string(),
        format_args!(
            "d_{}_{}_{}",
            base16::encode_lower(&ACCOUNT_3_ADDR_USER_1.as_bytes()),
            base16::encode_lower(&ACCOUNT_3_ADDR_USER_1.as_bytes()),
            GENESIS_VALIDATOR_STAKE.to_string()
        )
        .to_string(),
    ];

    let mut builder = InMemoryWasmTestBuilder::default();
    let result = builder
        .run_genesis(&utils::create_genesis_config(accounts, state_infos))
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be a genesis self-delegation
    let lookup_key_delegation = format!(
        "d_{}_{}_{}",
        base16::encode_lower(ACCOUNT_1_ADDR_DAPP_1.as_bytes()),
        base16::encode_lower(ACCOUNT_1_ADDR_DAPP_1.as_bytes()),
        GENESIS_VALIDATOR_STAKE
    );
    assert!(pos_contract
        .named_keys()
        .contains_key(&lookup_key_delegation));

    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_3_ADDR_USER_1.as_bytes()),
        GENESIS_VALIDATOR_STAKE
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    println!("Here we are");
    println!("0. send some tokens to system account");

    let token_transfer_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR_DAPP_1,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        (SYSTEM_ADDR, U512::from(SYSTEM_ACC_SUPPORT)),
    )
    .build();

    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(token_transfer_request)
        .expect_success()
        .commit()
        .finish();

    println!("Token sent!");

    let system_account = builder
        .get_account(SYSTEM_ADDR)
        .expect("system account should exist");
    let system_account_balance_actual = builder.get_purse_balance(system_account.main_purse());
    println!("system account balance: {}", system_account_balance_actual);

    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| { key.starts_with("t_") })
            .count(),
        1
    );

    // setup done. start distribute

    println!("2. distribute");
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder.step(StepRequestBuilder::default().build()).finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| { key.starts_with("ic_") })
            .count(),
        2
    );
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("ir_"))
            .count(),
        2
    );

    // Delegate some amount and try distribute
    println!("Delegate and try to step again");

    let delegate_request = ExecuteRequestBuilder::standard(
        ACCOUNT_2_ADDR_DAPP_2,
        CONTRACT_POS_VOTE,
        (
            String::from(METHOD_DELEGATE),
            ACCOUNT_1_ADDR_DAPP_1,
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

    println!("**** Dummy output from here ****");
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("ic_")
            })
            .count(),
        2
    );
    println!("**** Dummy output ends here ****");

    println!("Delegation done");
    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder.step(StepRequestBuilder::default().build()).finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    println!("**** Dummy output from here ****");
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("ic_")
            })
            .count(),
        2
    );
    println!("**** Dummy output ends here ****");

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

    println!("**** Dummy output from here ****");
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("ic_")
            })
            .count(),
        1
    );
    println!("**** Dummy output ends here ****");

    let account1_dapp_1 = builder
        .get_account(ACCOUNT_1_ADDR_DAPP_1)
        .expect("system account should exist");
    let system_account = builder
        .get_account(SYSTEM_ADDR)
        .expect("system account should exist");
    let account1_dapp_1_balance_actual = builder.get_purse_balance(account1_dapp_1.main_purse());
    let system_balance = builder.get_purse_balance(system_account.main_purse());

    println!("Account 1 balance: {}", account1_dapp_1_balance_actual);
    println!(
        "Initial: {}",
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE - SYSTEM_ACC_SUPPORT)
    );
    println!("System balance: {}", system_balance);

    println!("4. Reward");

    let reward_commission_request = ExecuteRequestBuilder::standard(
        ACCOUNT_1_ADDR_DAPP_1,
        CONTRACT_POS_VOTE,
        (String::from(METHOD_CLAIM_REWARD),),
    )
    .build();

    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder
        .exec(reward_commission_request)
        .expect_success()
        .commit()
        .finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    println!("**** Dummy output from here ****");
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| {
                println!("{}", key);
                key.starts_with("ir_")
            })
            .count(),
        2
    );
    println!("**** Dummy output ends here ****");

    let account1_dapp_1 = builder
        .get_account(ACCOUNT_1_ADDR_DAPP_1)
        .expect("system account should exist");
    let system_account = builder
        .get_account(SYSTEM_ADDR)
        .expect("system account should exist");
    let account1_dapp_1_balance_actual = builder.get_purse_balance(account1_dapp_1.main_purse());
    let system_balance = builder.get_purse_balance(system_account.main_purse());

    println!("Account 1 balance: {}", account1_dapp_1_balance_actual);
    println!(
        "Initial: {}",
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE - SYSTEM_ACC_SUPPORT)
    );
    println!("System balance: {}", system_balance);

    println!("5. Step again and check balance of the accounts");
    println!("Build Tx OK");

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _result = builder.step(StepRequestBuilder::default().build()).finish();

    println!("Exec OK");

    let account1_dapp_1 = builder
        .get_account(ACCOUNT_1_ADDR_DAPP_1)
        .expect("system account should exist");
    let system_account = builder
        .get_account(SYSTEM_ADDR)
        .expect("system account should exist");
    let account1_dapp_1_balance_actual = builder.get_purse_balance(account1_dapp_1.main_purse());
    let system_balance = builder.get_purse_balance(system_account.main_purse());

    println!("Account 1 balance: {}", account1_dapp_1_balance_actual);
    println!(
        "Initial: {}",
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE - SYSTEM_ACC_SUPPORT)
    );
    println!("System balance: {}", system_balance);
}
