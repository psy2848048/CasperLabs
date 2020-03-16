#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod constants;
mod contract_delegations;
mod contract_mint;
mod contract_queue;
mod contract_runtime;
mod contract_stakes;
<<<<<<< HEAD:execution-engine/contracts/hdac-system/pop/src/lib.rs
mod pop_contract;
=======
mod contract_votes;
mod hdac_pos_contract;
>>>>>>> feat: write contract_votes module and its error types:execution-engine/contracts/hdac-system/hdac-pos/src/lib.rs

use alloc::string::String;

use contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use proof_of_stake::ProofOfStake;
use types::{
    account::{PublicKey, PurseId},
    ApiError, CLValue, URef, U512,
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
        _ => {}
    }
}

#[cfg(not(feature = "lib"))]
#[no_mangle]
pub extern "C" fn call() {
    delegate();
}
