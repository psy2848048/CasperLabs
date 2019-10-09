#![no_std]

extern crate alloc;

extern crate contract_ffi;

use alloc::collections::btree_map::BTreeMap;
use alloc::prelude::v1::Vec;

use contract_ffi::contract_api::{self, Error};
use contract_ffi::unwrap_or_revert::UnwrapOrRevert;
use contract_ffi::value::account::PublicKey;

#[no_mangle]
pub extern "C" fn check_caller_ext() {
    let caller_public_key: PublicKey = contract_api::get_caller();
    contract_api::ret(&caller_public_key, &Vec::new())
}

#[no_mangle]
pub extern "C" fn call() {
    let known_public_key: PublicKey = contract_api::get_arg(0)
        .unwrap_or_revert_with(Error::MissingArgument)
        .unwrap_or_revert_with(Error::InvalidArgument);
    let caller_public_key: PublicKey = contract_api::get_caller();
    assert_eq!(
        caller_public_key, known_public_key,
        "caller public key was not known public key"
    );

    let pointer = contract_api::store_function_at_hash("check_caller_ext", BTreeMap::new());
    let subcall_public_key: PublicKey = contract_api::call_contract(pointer, &(), &Vec::new());
    assert_eq!(
        subcall_public_key, known_public_key,
        "subcall public key was not known public key"
    );
}