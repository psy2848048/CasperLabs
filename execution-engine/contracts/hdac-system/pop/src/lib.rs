#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod constants;
mod math;
mod contract_delegations;
mod contract_economy;
mod contract_mint;
mod contract_queue;
mod contract_runtime;
mod contract_stakes;
mod contract_votes;
mod pop_contract;

use alloc::string::String;

use contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use proof_of_stake::ProofOfStake;
use types::{
    account::{PublicKey, PurseId},
    ApiError, CLValue, Key, URef, U512,
};

pub use crate::pop_contract::ProofOfProfessionContract;

use crate::constants::methods;

pub fn delegate() {
    let pop_contract = ProofOfProfessionContract;

    let method_name: String = runtime::get_arg(0)
        .unwrap_or_revert_with(ApiError::MissingArgument)
        .unwrap_or_revert_with(ApiError::InvalidArgument);

    match method_name.as_str() {
        // Type of this method: `fn bond(amount: U512, purse: URef)`
        methods::METHOD_BOND => {
            let validator = runtime::get_caller();
            let amount: U512 = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let source_uref: URef = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract
                .delegate(validator, validator, amount, source_uref)
                .unwrap_or_revert();
        }
        // Type of this method: `fn unbond(amount: Option<U512>)`
        methods::METHOD_UNBOND => {
            let validator = runtime::get_caller();
            let maybe_amount = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract
                .undelegate(validator, validator, maybe_amount)
                .unwrap_or_revert();
        }
        // Type of this method: `fn step()`
        methods::METHOD_STEP => {
            // This is called by the system in every block.
            pop_contract.step().unwrap_or_revert();
        }
        // Type of this method: `fn get_payment_purse() -> PurseId`
        methods::METHOD_GET_PAYMENT_PURSE => {
            let rights_controlled_purse = pop_contract.get_payment_purse().unwrap_or_revert();
            let return_value = CLValue::from_t(rights_controlled_purse).unwrap_or_revert();
            runtime::ret(return_value);
        }
        // Type of this method: `fn set_refund_purse(purse_id: PurseId)`
        methods::METHOD_SET_REFUND_PURSE => {
            let purse_id: PurseId = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract.set_refund_purse(purse_id).unwrap_or_revert();
        }
        // Type of this method: `fn get_refund_purse() -> PurseId`
        methods::METHOD_GET_REFUND_PURSE => {
            // We purposely choose to remove the access rights so that we do not
            // accidentally give rights for a purse to some contract that is not
            // supposed to have it.
            let maybe_purse_uref = pop_contract.get_refund_purse().unwrap_or_revert();
            let return_value = CLValue::from_t(maybe_purse_uref).unwrap_or_revert();
            runtime::ret(return_value);
        }
        // Type of this method: `fn finalize_payment()`
        methods::METHOD_FINALIZE_PAYMENT => {
            let amount_spent: U512 = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let account: PublicKey = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract
                .finalize_payment(amount_spent, account)
                .unwrap_or_revert();
        }
        // Type of this method: `fn delegate(validator: PublicKey, amount: U512, src_purse_uref:
        // URef)`
        methods::METHOD_DELEGATE => {
            let delegator = runtime::get_caller();
            let validator: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: U512 = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let source_uref: URef = runtime::get_arg(3)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract
                .delegate(delegator, validator, amount, source_uref)
                .unwrap_or_revert();
        }
        // Type of this method: `fn undelegate(validator: PublicKey, amount: Option<U512>)`
        methods::METHOD_UNDELEGATE => {
            let delegator: PublicKey = runtime::get_caller();
            let validator: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let shares: Option<U512> = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract
                .undelegate(delegator, validator, shares)
                .unwrap_or_revert();
        }
        // Type of this method: `fn redelegate(src_validator: PublicKey, dest_validator: PublicKey,
        // amount: U512)`
        methods::METHOD_REDELEGATE => {
            let delegator: PublicKey = runtime::get_caller();
            let src_validator: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let dest_validator: PublicKey = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let shares: U512 = runtime::get_arg(3)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract
                .redelegate(delegator, src_validator, dest_validator, shares)
                .unwrap_or_revert();
        }
        methods::METHOD_VOTE => {
            let user: PublicKey = runtime::get_caller();
            let dapp: Key = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: U512 = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract.vote(user, dapp, amount).unwrap_or_revert();
        }
        methods::METHOD_UNVOTE => {
            let user: PublicKey = runtime::get_caller();
            let dapp: Key = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: Option<U512> = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract.unvote(user, dapp, amount).unwrap_or_revert();
        }
        methods::METHOD_WRITE_GENESIS_TOTAL_SUPPLY => {
            let maybe_system_user: PublicKey = runtime::get_caller();
            // system user: PublicKey([0, 0, 0, ... , 0]) 32 of 0s
            if maybe_system_user != PublicKey::new([0u8; 32]) {
                runtime::revert(ApiError::NoAccessRights);
            }
            let genesis_total_supply: U512 = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            pop_contract.write_genesis_total_supply(&genesis_total_supply).unwrap_or_revert();
        }
        methods::METHOD_DISTRIBUTE => {
            let maybe_system_user: PublicKey = runtime::get_caller();
            // system user: PublicKey([0, 0, 0, ... , 0]) 32 of 0s
            if maybe_system_user != PublicKey::new([0u8; 32]) {
                runtime::revert(ApiError::NoAccessRights);
            }
            pop_contract.distribute().unwrap_or_revert();
        }
        methods::METHOD_CLAIM_COMMISSION => {
            let validator: PublicKey = runtime::get_caller();
            pop_contract.claim_commission(&validator).unwrap_or_revert();
        }
        methods::METHOD_CLAIM_REWARD => {
            let user: PublicKey = runtime::get_caller();
            pop_contract.claim_reward(&user).unwrap_or_revert();
        }
        _ => {}
    }
}

#[cfg(not(feature = "lib"))]
#[no_mangle]
pub extern "C" fn call() {
    delegate();
}
