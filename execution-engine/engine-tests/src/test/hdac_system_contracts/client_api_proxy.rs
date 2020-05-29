use num_traits::identities::Zero;

use engine_core::engine_state::{
    genesis::{GenesisAccount, POS_BONDING_PURSE},
    SYSTEM_ACCOUNT_ADDR,
};
use engine_shared::{motes::Motes, stored_value::StoredValue};
use types::{account::PublicKey, Key, U512};

use engine_test_support::{
    internal::{
        utils, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder,
        StepRequestBuilder, DEFAULT_ACCOUNT_KEY, DEFAULT_GENESIS_CONFIG, DEFAULT_PAYMENT,
    },
    DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE,
};

const ACCOUNT_1_ADDR: PublicKey = PublicKey::ed25519_from([1u8; 32]);

const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;

const TRANSFER_TO_ACCOUNT_METHOD: &str = "transfer_to_account";
const BOND_METHOD: &str = "bond";
const UNBOND_METHOD: &str = "unbond";
const DELEGATE_METHOD: &str = "delegate";
const UNDELEGATE_METHOD: &str = "undelegate";
const REDELEGATE_METHOD: &str = "redelegate";
const VOTE_METHOD: &str = "vote";
const UNVOTE_METHOD: &str = "unvote";
const CLAIM_COMMISSION_METHOD: &str = "claim_commission";
const CLAIM_REWARD_METHOD: &str = "claim_reward";

fn get_client_api_proxy_hash(builder: &InMemoryWasmTestBuilder) -> [u8; 32] {
    // query client_api_proxy_hash from SYSTEM_ACCOUNT
    let system_account = match builder
        .query(None, Key::Account(SYSTEM_ACCOUNT_ADDR), &[])
        .expect("should query system account")
    {
        StoredValue::Account(account) => account,
        _ => panic!("should get an account"),
    };

    system_account
        .named_keys()
        .get("client_api_proxy")
        .expect("should get client_api_proxy key")
        .into_hash()
        .expect("should be hash")
}

#[ignore]
#[test]
fn should_invoke_successful_transfer_to_account() {
    let transferred_amount = U512::from(1000);

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_GENESIS_CONFIG).commit();

    let client_api_proxy_hash = get_client_api_proxy_hash(&builder);

    // transfer to ACCOUNT_1_ADDR with transferred_amount
    let exec_request = ExecuteRequestBuilder::contract_call_by_hash(
        DEFAULT_ACCOUNT_ADDR,
        client_api_proxy_hash,
        (
            TRANSFER_TO_ACCOUNT_METHOD,
            ACCOUNT_1_ADDR,
            transferred_amount,
        ),
    )
    .build();

    let test_result = builder.exec_commit_finish(exec_request);

    let account_1 = test_result
        .builder()
        .get_account(ACCOUNT_1_ADDR)
        .expect("should get account 1");

    let balance = test_result
        .builder()
        .get_purse_balance(account_1.main_purse());

    assert_eq!(balance, transferred_amount);
}

#[ignore]
#[test]
fn should_invoke_successful_standard_payment() {
    // run genesis
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_GENESIS_CONFIG).commit();

    let client_api_proxy_hash = get_client_api_proxy_hash(&builder);

    // transfer 1 from DEFAULT_ACCOUNT to ACCOUNT_1
    let transferred_amount = 1;
    let exec_request = {
        let deploy = DeployItemBuilder::new()
            .with_address(DEFAULT_ACCOUNT_ADDR)
            .with_deploy_hash([1; 32])
            .with_session_code(
                "transfer_purse_to_account.wasm",
                (ACCOUNT_1_ADDR, U512::from(transferred_amount)),
            )
            .with_stored_payment_hash(
                client_api_proxy_hash.to_vec(),
                ("standard_payment", *DEFAULT_PAYMENT),
            )
            .with_authorization_keys(&[DEFAULT_ACCOUNT_KEY])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };
    let transfer_result = builder.exec_commit_finish(exec_request);
    let default_account = transfer_result
        .builder()
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should get genesis account");
    let modified_balance = transfer_result
        .builder()
        .get_purse_balance(default_account.main_purse());
    let initial_balance = U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE);

    assert_ne!(
        modified_balance, initial_balance,
        "balance should be less than initial balance"
    );

    let total_consumed = *DEFAULT_PAYMENT + U512::from(transferred_amount);
    let tally = total_consumed + modified_balance;

    assert_eq!(
        initial_balance, tally,
        "no net resources should be gained or lost post-distribution"
    );
}

#[ignore]
#[test]
fn should_invoke_successful_bond_and_unbond() {
    const BOND_AMOUNT: u64 = 1_000_000;

    // only DEFAULT_ACCOUNT is in initial validator queue.
    let accounts: Vec<GenesisAccount> = vec![GenesisAccount::new(
        DEFAULT_ACCOUNT_ADDR,
        Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
        Motes::new(BOND_AMOUNT.into()),
    )];
    let state_infos = vec![format_args!(
        "d_{}_{}_{}",
        base16::encode_lower(&DEFAULT_ACCOUNT_ADDR.as_bytes()),
        base16::encode_lower(&DEFAULT_ACCOUNT_ADDR.as_bytes()),
        BOND_AMOUNT.to_string()
    )
    .to_string()];
    let genesis_config = utils::create_genesis_config(accounts, state_infos);
    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&genesis_config)
        .commit()
        .finish();

    let client_api_proxy_hash = get_client_api_proxy_hash(result.builder());

    // Transfer to ACCOUNT_1_ADDR request
    let exec_request_transfer = ExecuteRequestBuilder::contract_call_by_hash(
        DEFAULT_ACCOUNT_ADDR,
        client_api_proxy_hash,
        (
            TRANSFER_TO_ACCOUNT_METHOD,
            ACCOUNT_1_ADDR,
            *DEFAULT_PAYMENT * 5,
        ),
    )
    .build();
    // Bonding request
    let exec_request_bonding = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR,
        client_api_proxy_hash,
        (BOND_METHOD, U512::from(BOND_AMOUNT)),
    )
    .build();
    let bonding_result = InMemoryWasmTestBuilder::from_result(result)
        .exec(exec_request_transfer)
        .expect_success()
        .commit()
        .exec(exec_request_bonding)
        .expect_success()
        .commit()
        .finish();

    let pos_contract = bonding_result.builder().get_pos_contract();

    let lookup_key = format!(
        "v_{}_{}",
        base16::encode_lower(&ACCOUNT_1_ADDR.as_bytes()),
        BOND_AMOUNT
    );
    assert!(pos_contract.named_keys().contains_key(&lookup_key));

    // Unbonding request
    let exec_request_unbonding = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR,
        client_api_proxy_hash,
        (UNBOND_METHOD, None as Option<U512>), // None means unbond all the amount
    )
    .build();
    let unbonding_result = InMemoryWasmTestBuilder::from_result(bonding_result)
        .exec(exec_request_unbonding)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let pos_contract = unbonding_result.builder().get_pos_contract();

    // ensure that ACCOUNT_1_ADDR is not in validator queue.
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with(&format!(
                "v_{}",
                base16::encode_lower(ACCOUNT_1_ADDR.as_bytes())
            )))
            .count(),
        0
    );
    // only genesis validator is still in the queue
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("v_"))
            .count(),
        1
    );
}

#[ignore]
#[test]
fn should_invoke_successful_delegation_methods() {
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

    let state_infos = vec![
        format_args!(
            "d_{}_{}_{}",
            base16::encode_lower(&ACCOUNT_1_ADDR.as_bytes()),
            base16::encode_lower(&ACCOUNT_1_ADDR.as_bytes()),
            GENESIS_VALIDATOR_STAKE.to_string()
        )
        .to_string(),
        format_args!(
            "d_{}_{}_{}",
            base16::encode_lower(&ACCOUNT_2_ADDR.as_bytes()),
            base16::encode_lower(&ACCOUNT_2_ADDR.as_bytes()),
            GENESIS_VALIDATOR_STAKE.to_string()
        )
        .to_string(),
    ];

    let genesis_config = utils::create_genesis_config(accounts, state_infos);
    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&genesis_config)
        .commit()
        .finish();

    let client_api_proxy_hash = get_client_api_proxy_hash(result.builder());

    // ACCOUNT_3 delegate to ACCOUNT_1
    let delegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR,
        client_api_proxy_hash,
        (
            DELEGATE_METHOD,
            ACCOUNT_1_ADDR,
            U512::from(ACCOUNT_3_DELEGATE_AMOUNT),
        ),
    )
    .build();

    // ACCOUNT_3 redelegate from ACCOUNT_1 to ACCOUNT_2
    let redelegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR,
        client_api_proxy_hash,
        (
            REDELEGATE_METHOD,
            ACCOUNT_1_ADDR,
            ACCOUNT_2_ADDR,
            U512::from(ACCOUNT_3_REDELEGATE_AMOUNT),
        ),
    )
    .build();

    // ACCOUNT_3 undelegate all from ACCOUNT_1
    let undelegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR,
        client_api_proxy_hash,
        (UNDELEGATE_METHOD, ACCOUNT_1_ADDR, None as Option<U512>),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    builder
        .exec(delegate_request)
        .expect_success()
        .commit()
        .exec(redelegate_request)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .expect_success()
        .exec(undelegate_request)
        .expect_success()
        .commit()
        .step(StepRequestBuilder::default().build())
        .finish();

    let pos_uref = builder.get_pos_contract_uref();
    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("must get pos contract");

    // Validate delegations
    let expected_delegation_1 = format!(
        "d_{}_{}_{}",
        base16::encode_lower(ACCOUNT_3_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        ACCOUNT_3_REDELEGATE_AMOUNT
    );
    let delegation_key_that_should_not_exist = format!(
        "d_{}_{}",
        base16::encode_lower(ACCOUNT_3_ADDR.as_bytes()),
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes())
    );
    assert!(pos_contract
        .named_keys()
        .contains_key(&expected_delegation_1));
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with(&delegation_key_that_should_not_exist))
            .count(),
        0
    );
    // There are 2 self delegations and one delegation d_{ACCOUNT_3}_{ACCOUNT_2}_{REDELEGATE_AMOUNT}
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("d_"))
            .count(),
        3
    );

    // Validate stakes
    let expected_stakes_1 = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_1_ADDR.as_bytes()),
        GENESIS_VALIDATOR_STAKE
    );
    let expected_stakes_2 = format!(
        "v_{}_{}",
        base16::encode_lower(ACCOUNT_2_ADDR.as_bytes()),
        GENESIS_VALIDATOR_STAKE + ACCOUNT_3_REDELEGATE_AMOUNT
    );

    assert!(pos_contract.named_keys().contains_key(&expected_stakes_1));
    assert!(pos_contract.named_keys().contains_key(&expected_stakes_2));

    // There should be only 2 stakes.
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("v_"))
            .count(),
        2
    );

    // Validate pos_bonding_purse balance
    let pos_bonding_purse_balance = {
        let purse_id = pos_contract
            .named_keys()
            .get(POS_BONDING_PURSE)
            .and_then(Key::as_uref)
            .expect("should find PoS payment purse");

        builder.get_purse_balance(*purse_id)
    };
    assert_eq!(
        pos_bonding_purse_balance,
        (GENESIS_VALIDATOR_STAKE * 2 + ACCOUNT_3_REDELEGATE_AMOUNT).into()
    );
}

#[ignore]
#[test]
fn should_invoke_successful_vote_and_unvote() {
    const ACCOUNT_1_ADDR_DAPP_1: PublicKey = PublicKey::ed25519_from([1u8; 32]);
    const ACCOUNT_2_ADDR_DAPP_2: PublicKey = PublicKey::ed25519_from([2u8; 32]);
    const ACCOUNT_3_ADDR_USER_1: PublicKey = PublicKey::ed25519_from([3u8; 32]);
    const ACCOUNT_4_ADDR_USER_2: PublicKey = PublicKey::ed25519_from([4u8; 32]);
    const ACCOUNT_5_ADDR_USER_3: PublicKey = PublicKey::ed25519_from([5u8; 32]);

    const GENESIS_VALIDATOR_STAKE: u64 = 50_000;
    const ACCOUNT_3_VOTE_AMOUNT: u64 = 10_000;

    let accounts = vec![
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

    let client_api_proxy_hash = get_client_api_proxy_hash(result.builder());

    // setup done. start testing
    // execute vote
    // vote by ACCOUNT_3_ADDR_USER_1 to ACCOUNT_1_ADDR_DAPP_1

    let vote_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR_USER_1,
        client_api_proxy_hash,
        (
            String::from(VOTE_METHOD),
            Key::Hash(ACCOUNT_1_ADDR_DAPP_1.value()),
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

    // execute second vote with user 1 to another dapp
    let vote_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR_USER_1,
        client_api_proxy_hash,
        (
            String::from(VOTE_METHOD),
            Key::Hash(ACCOUNT_2_ADDR_DAPP_2.value()),
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
    let unvote_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_3_ADDR_USER_1,
        client_api_proxy_hash,
        (
            String::from(UNVOTE_METHOD),
            Key::Hash(ACCOUNT_1_ADDR_DAPP_1.value()),
            None::<U512>,
        ),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let _result = builder
        .exec(unvote_request)
        .expect_success()
        .commit()
        .finish();

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be still 2 delegations
    assert_eq!(
        pos_contract
            .named_keys()
            .iter()
            .filter(|(key, _)| key.starts_with("a_"))
            .count(),
        1
    );
}

#[ignore]
#[test]
fn should_invoke_successful_step() {
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

    let client_api_proxy_hash = get_client_api_proxy_hash(result.builder());

    let token_transfer_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR_DAPP_1,
        client_api_proxy_hash,
        (
            String::from(TRANSFER_TO_ACCOUNT_METHOD),
            SYSTEM_ADDR,
            U512::from(SYSTEM_ACC_SUPPORT),
        ),
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

    println!("2. distribute");
    let mut builder = InMemoryWasmTestBuilder::from_result(result);
    let result = builder.step(StepRequestBuilder::default().build()).finish();

    println!("Exec OK");

    let pos_contract = builder
        .get_contract(pos_uref.remove_access_rights())
        .expect("should have contract");

    // there should be a still only one validator.
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

    let delegate_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_2_ADDR_DAPP_2,
        client_api_proxy_hash,
        (
            String::from(DELEGATE_METHOD),
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

    let claim_commission_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR_DAPP_1,
        client_api_proxy_hash,
        (String::from(CLAIM_COMMISSION_METHOD),),
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

    let reward_commission_request = ExecuteRequestBuilder::contract_call_by_hash(
        ACCOUNT_1_ADDR_DAPP_1,
        client_api_proxy_hash,
        (String::from(CLAIM_REWARD_METHOD),),
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
