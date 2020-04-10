#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, string::String};
use core::fmt::Write;

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::{PublicKey, PurseId},
    system_contract_errors::mint,
    AccessRights, ApiError, CLValue, ContractRef, Key, URef, U512,
};

const PLACEHOLDER_KEY: Key = Key::Hash([0u8; 32]);
const POS_BONDING_PURSE: &str = "pos_bonding_purse";
const POS_PAYMENT_PURSE: &str = "pos_payment_purse";
const POS_REWARDS_PURSE: &str = "pos_rewards_purse";
const POS_FUNCTION_NAME: &str = "pos_ext";
const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;

#[repr(u32)]
enum Args {
    MintURef = 0,
    GenesisValidators = 1,
}

#[no_mangle]
pub extern "C" fn pos_ext() {
    pop::delegate();
}

#[no_mangle]
pub extern "C" fn call() {
    let mint_uref: URef = runtime::get_arg(Args::MintURef as u32)
        .unwrap_or_revert_with(ApiError::MissingArgument)
        .unwrap_or_revert_with(ApiError::InvalidArgument);
    let mint = ContractRef::URef(URef::new(mint_uref.addr(), AccessRights::READ));

    let genesis_validators: BTreeMap<PublicKey, U512> =
        runtime::get_arg(Args::GenesisValidators as u32)
            .unwrap_or_revert_with(ApiError::MissingArgument)
            .unwrap_or_revert_with(ApiError::InvalidArgument);
    // Add genesis validators to PoP contract object.
    // For now, we are storing validators in `named_keys` map of the PoP contract
    // in the form: key: "v_{validator_pk}_{validator_stake}", value: doesn't
    // matter.
    let mut named_keys: BTreeMap<String, Key> = genesis_validators
        .iter()
        .map(|(pub_key, balance)| {
            let key_bytes = pub_key.value();
            let mut hex_key = String::with_capacity(64);
            for byte in &key_bytes[..32] {
                write!(hex_key, "{:02x}", byte).unwrap();
            }
            let mut uref = String::new();
            uref.write_fmt(format_args!("v_{}_{}", hex_key, balance))
                .unwrap();
            uref
        })
        .map(|key| (key, PLACEHOLDER_KEY))
        .collect();

    // Insert genesis validator's delegations.
    // We also store delegations in the form key:
    // "d_{delegator_pk}_{validator_pk}_{delegation_amount}", value: doesn't matter
    genesis_validators
        .iter()
        .map(|(pub_key, balance)| {
            let key_bytes = pub_key.value();
            let mut hex_key = String::with_capacity(64);
            for byte in &key_bytes[..32] {
                write!(hex_key, "{:02x}", byte).unwrap();
            }
            let mut uref = String::new();
            uref.write_fmt(format_args!("d_{}_{}_{}", hex_key, hex_key, balance))
                .unwrap();
            uref
        })
        .for_each(|key| {
            named_keys.insert(key, PLACEHOLDER_KEY);
        });

    // Insert total supply
    let mut total_supply_uref = String::new();
    total_supply_uref
        .write_fmt(format_args!(
            "t_{}",
            U512::from(2_000_000_000_u64) * U512::from(BIGSUN_TO_HDAC)
        ))
        .unwrap();
    named_keys.insert(total_supply_uref, PLACEHOLDER_KEY);

    let total_bonds: U512 = genesis_validators.values().fold(U512::zero(), |x, y| x + y);

    let bonding_purse = mint_purse(&mint, total_bonds);
    let payment_purse = mint_purse(&mint, U512::zero());
    // let rewards_purse = mint_purse(&mint, U512::zero());
    // Charge unreachable amount of token into inaccessible wallet
    let rewards_purse = mint_purse(
        &mint,
        U512::from(999_999_999_999_u64) * U512::from(BIGSUN_TO_HDAC),
    );

    // Include PoP purses in its named_keys
    [
        (POS_BONDING_PURSE, bonding_purse.value()),
        (POS_PAYMENT_PURSE, payment_purse.value()),
        (POS_REWARDS_PURSE, rewards_purse.value()),
    ]
    .iter()
    .for_each(|(name, uref)| {
        named_keys.insert(String::from(*name), Key::URef(*uref));
    });

    let uref: URef = storage::store_function(POS_FUNCTION_NAME, named_keys)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedContractRefVariant);
    let return_value = CLValue::from_t(uref).unwrap_or_revert();

    // store a contract which serves as proxy for commonly used client apis.
    client_api_proxy::deploy_client_api_proxy();

    runtime::ret(return_value);
}

fn mint_purse(mint: &ContractRef, amount: U512) -> PurseId {
    let result: Result<URef, mint::Error> = runtime::call_contract(mint.clone(), ("mint", amount));

    result.map(PurseId::new).unwrap_or_revert()
}
