#![no_std]

extern crate alloc;

use alloc::string::String;

use contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::{PublicKey, PurseId},
    ApiError, ContractRef, U512,
};

#[repr(u16)]
enum Error {
    UnknownCommand,
}

fn bond(pos: &ContractRef, amount: &U512, source: PurseId) {
    runtime::call_contract::<_, ()>(pos.clone(), (POS_BOND, *amount, source));
}

fn unbond(pos: &ContractRef, amount: Option<U512>) {
    runtime::call_contract::<_, ()>(pos.clone(), (POS_UNBOND, amount));
}

fn delegate(pos: &ContractRef, validator: &PublicKey, amount: &U512, source: PurseId) {
    runtime::call_contract::<_, ()>(pos.clone(), (POS_DELEGATE, *validator, *amount, source));
}

fn undelegate(pos: &ContractRef, validator: &PublicKey, amount: Option<U512>) {
    runtime::call_contract::<_, ()>(pos.clone(), (POS_UNDELEGATE, *validator, amount));
}

fn redelegate(
    pos: &ContractRef,
    src_validator: &PublicKey,
    dest_validator: &PublicKey,
    amount: &U512,
) {
    runtime::call_contract::<_, ()>(
        pos.clone(),
        (POS_REDELEGATE, *src_validator, *dest_validator, *amount),
    );
}

fn vote(pos: &ContractRef, dapp_key: PublicKey, user_key: PublicKey, amount: &U512) {
    runtime::call_contract::<_, ()>(pos.clone(), (POS_BOND, dapp_key, user_key, *amount));
}

fn unvote(pos: &ContractRef, dapp_key: PublicKey, user_key: PublicKey, amount: &U512) {
    runtime::call_contract::<_, ()>(pos.clone(), (POS_UNBOND, dapp_key, user_key, *amount));
}

const POS_BOND: &str = "bond";
const POS_UNBOND: &str = "unbond";
const POS_DELEGATE: &str = "delegate";
const POS_UNDELEGATE: &str = "undelegate";
const POS_REDELEGATE: &str = "redelegate";
const POS_VOTE: &str = "vote";
const POS_UNVOTE: &str = "unvote";

#[no_mangle]
pub extern "C" fn call() {
    let pos_pointer = system::get_proof_of_stake();

    let command: String = runtime::get_arg(0)
        .unwrap_or_revert_with(ApiError::MissingArgument)
        .unwrap_or_revert_with(ApiError::InvalidArgument);

    match command.as_ref() {
        POS_BOND => {
            let amount: U512 = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let p1 = system::create_purse();

            system::transfer_from_purse_to_purse(account::get_main_purse(), p1, amount)
                .unwrap_or_revert();

            bond(&pos_pointer, &amount, p1);
        }
        POS_UNBOND => {
            let amount: Option<U512> = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);

            unbond(&pos_pointer, amount);
        }
        POS_DELEGATE => {
            let validator: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: U512 = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let p1 = system::create_purse();

            system::transfer_from_purse_to_purse(account::get_main_purse(), p1, amount)
                .unwrap_or_revert();

            delegate(&pos_pointer, &validator, &amount, p1);
        }
        POS_UNDELEGATE => {
            let validator: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: Option<U512> = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            undelegate(&pos_pointer, &validator, amount);
        }
        POS_REDELEGATE => {
            let src_validator: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let dest_validator: PublicKey = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: U512 = runtime::get_arg(3)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            redelegate(&pos_pointer, &src_validator, &dest_validator, &amount);
        }
        POS_VOTE => {
            let user: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let dapp: PublicKey = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: U512 = runtime::get_arg(3)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            vote(&pos_pointer, user, dapp, &amount)
        }
        POS_UNVOTE => {
            let user: PublicKey = runtime::get_arg(1)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let dapp: PublicKey = runtime::get_arg(2)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            let amount: U512 = runtime::get_arg(3)
                .unwrap_or_revert_with(ApiError::MissingArgument)
                .unwrap_or_revert_with(ApiError::InvalidArgument);
            unvote(&pos_pointer, user, dapp, &amount)
        }
        _ => runtime::revert(ApiError::User(Error::UnknownCommand as u16)),
    }
}
