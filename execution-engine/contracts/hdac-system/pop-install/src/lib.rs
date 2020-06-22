#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, string::String};

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::PublicKey, system_contract_errors::mint, AccessRights, ApiError, CLValue, ContractRef,
    Key, URef, U512,
};

const POS_BONDING_PURSE: &str = "pos_bonding_purse";
const POS_PAYMENT_PURSE: &str = "pos_payment_purse";
const POS_REWARDS_PURSE: &str = "pos_rewards_purse";
const POS_COMMISSION_PURSE: &str = "pos_commission_purse";
const POS_COMMUNITY_PURSE: &str = "pos_community_purse";
const POP_FUNCTION_NAME: &str = "pop_ext";
const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;

#[repr(u32)]
enum Args {
    MintURef = 0,
    GenesisValidators = 1,
}

#[no_mangle]
pub extern "C" fn pop_ext() {
    pop::delegate();
}

#[no_mangle]
pub extern "C" fn call() {
    let mint_uref: URef = runtime::get_arg(Args::MintURef as u32)
        .unwrap_or_revert_with(ApiError::MissingArgument)
        .unwrap_or_revert_with(ApiError::InvalidArgument);

    // TODO: split this into genesis_stakes and genesis_delegation
    let genesis_validators: BTreeMap<PublicKey, U512> =
        runtime::get_arg(Args::GenesisValidators as u32)
            .unwrap_or_revert_with(ApiError::MissingArgument)
            .unwrap_or_revert_with(ApiError::InvalidArgument);

    let named_keys = build_pop_named_keys(mint_uref, &genesis_validators);

    let pop_uref: URef = storage::store_function(POP_FUNCTION_NAME, named_keys)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedContractRefVariant);
    let return_value = CLValue::from_t(pop_uref).unwrap_or_revert();

    let pop = ContractRef::URef(URef::new(pop_uref.addr(), AccessRights::READ));
    runtime::call_contract::<_, ()>(
        pop,
        (
            "install_genesis_states",
            U512::from(2_000_000_000_u64) * U512::from(BIGSUN_TO_HDAC), // total_mint_supply
            genesis_validators,                                         /* , genesis_delegations */
        ),
    );

    // store a contract which serves as proxy for commonly used client apis.
    client_api_proxy::deploy_client_api_proxy();

    runtime::ret(return_value);
}

fn build_pop_named_keys(
    mint_uref: URef,
    genesis_validators: &BTreeMap<PublicKey, U512>,
) -> BTreeMap<String, Key> {
    let mint = ContractRef::URef(URef::new(mint_uref.addr(), AccessRights::READ));
    let mut named_keys = BTreeMap::<String, Key>::default();

    let total_bonds = genesis_validators.values().fold(U512::zero(), |x, y| x + y);
    let bonding_purse = mint_purse(&mint, total_bonds);
    let payment_purse = mint_purse(&mint, U512::zero());
    let rewards_purse = mint_purse(&mint, U512::zero());
    let commission_purse = mint_purse(&mint, U512::zero());
    let community_purse = mint_purse(&mint, U512::zero());

    // Include PoP purses in its named_keys
    [
        (POS_BONDING_PURSE, bonding_purse),
        (POS_PAYMENT_PURSE, payment_purse),
        (POS_REWARDS_PURSE, rewards_purse),
        (POS_COMMISSION_PURSE, commission_purse),
        (POS_COMMUNITY_PURSE, community_purse),
    ]
    .iter()
    .for_each(|(name, uref)| {
        named_keys.insert(String::from(*name), Key::URef(*uref));
    });

    named_keys
}

fn mint_purse(mint: &ContractRef, amount: U512) -> URef {
    let result: Result<URef, mint::Error> = runtime::call_contract(mint.clone(), ("mint", amount));

    result.unwrap_or_revert()
}
