#![no_std]

extern crate alloc;

use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::Write;

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::PublicKey,
    system_contract_errors::{mint, pos::Error},
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
    StateInformations = 2,
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
    let mut validators: BTreeMap<String, U512> = BTreeMap::new();
    let mut named_keys: BTreeMap<String, Key> = genesis_validators
        .iter()
        .map(|(pub_key, balance)| {
            let key_bytes = pub_key.value();
            let mut hex_key = String::with_capacity(64);
            for byte in &key_bytes[..32] {
                write!(hex_key, "{:02x}", byte).unwrap();
            }
            validators.insert(hex_key.clone(), *balance);
            let mut uref = String::new();
            uref.write_fmt(format_args!("v_{}_{}", hex_key, balance))
                .unwrap();
            uref
        })
        .map(|key| (key, PLACEHOLDER_KEY))
        .collect();

    let mut delegators: BTreeMap<String, U512> = BTreeMap::new();
    let mut voters: BTreeMap<String, U512> = BTreeMap::new();
    let mut total_delegates: U512 = U512::zero();
    // Insert genesis state information.
    // We also store in the form key:
    let state_informations: Vec<String> = runtime::get_arg(Args::StateInformations as u32)
        .unwrap_or_revert_with(ApiError::MissingArgument)
        .unwrap_or_revert_with(ApiError::InvalidArgument);
    state_informations.iter().for_each(|key| {
        let split_key: Vec<&str> = key.split('_').collect();
        match split_key[0] {
            "c" => {
                if split_key.len() != 3 {
                    runtime::revert(Error::CommissionKeyDeserializationFailed);
                }
                if split_key[1].len() != 64 {
                    runtime::revert(Error::CommissionKeyDeserializationFailed);
                }
            }
            "r" => {
                if split_key.len() != 3 {
                    runtime::revert(Error::RewardKeyDeserializationFailed);
                }
                if split_key[1].len() != 64 {
                    runtime::revert(Error::RewardKeyDeserializationFailed);
                }
            }
            "d" => {
                if split_key.len() != 4 {
                    runtime::revert(Error::DelegationsKeyDeserializationFailed);
                }
                if split_key[1].len() != 64 && split_key[2].len() != 64 {
                    runtime::revert(Error::DelegationsKeyDeserializationFailed);
                }
                match U512::from_dec_str(split_key[3]) {
                    Ok(amount) => {
                        if !validators.contains_key(split_key[2]) {
                            runtime::revert(Error::DelegationsKeyDeserializationFailed);
                        }
                        match validators.get_mut(split_key[2]) {
                            Some(a) => *a -= amount,
                            None => runtime::revert(Error::DelegationsNotFound),
                        }

                        match delegators.get_mut(split_key[1]) {
                            Some(a) => *a += amount,
                            None => {
                                delegators.insert(split_key[1].to_string(), amount);
                            }
                        };

                        total_delegates += amount;
                    }
                    Err(_) => runtime::revert(Error::DelegationsDeserializationFailed),
                }
            }
            "a" => {
                if split_key.len() != 4 {
                    runtime::revert(Error::VoteKeyDeserializationFailed);
                }
                if split_key[1].len() != 64 {
                    runtime::revert(Error::VoteKeyDeserializationFailed);
                }
                if !((split_key[2].len() == 66) || (split_key[2].len() == 68)) {
                    runtime::revert(Error::VoteKeyDeserializationFailed);
                }

                match U512::from_dec_str(split_key[3]) {
                    Ok(amount) => {
                        match voters.get_mut(split_key[1]) {
                            Some(a) => *a += amount,
                            None => {
                                voters.insert(split_key[1].to_string(), amount);
                            }
                        };
                    }
                    Err(_) => runtime::revert(Error::UintParsingError),
                };
            }
            _ => runtime::revert(Error::VotesDeserializationFailed),
        }
        named_keys.insert(key.to_string(), PLACEHOLDER_KEY);
    });

    // check validate state information
    for (_, amount) in validators.iter() {
        if *amount != U512::zero() {
            runtime::revert(Error::NotMatchedTotalBondAndDelegate);
        }
    }

    for (voter_address, voter_amount) in voters.iter() {
        match delegators.get(voter_address) {
            Some(a) => {
                if *a < *voter_amount {
                    runtime::revert(Error::VoteTooLarge);
                }
            }
            None => runtime::revert(Error::VotesNotFound),
        }
    }

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

    if total_bonds != total_delegates {
        runtime::revert(Error::NotMatchedTotalBondAndDelegate);
    }

    let bonding_purse = mint_purse(&mint, total_bonds);
    let payment_purse = mint_purse(&mint, U512::zero());
    // let rewards_purse = mint_purse(&mint, U512::zero());
    // Charge unreachable amount of token into inaccessible wallet
    let rewards_purse = mint_purse(&mint, U512::zero());

    // Include PoP purses in its named_keys
    [
        (POS_BONDING_PURSE, bonding_purse),
        (POS_PAYMENT_PURSE, payment_purse),
        (POS_REWARDS_PURSE, rewards_purse),
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

fn mint_purse(mint: &ContractRef, amount: U512) -> URef {
    let result: Result<URef, mint::Error> = runtime::call_contract(mint.clone(), ("mint", amount));

    result.unwrap_or_revert()
}
