#![no_std]

#[macro_use]
extern crate alloc;

extern crate contract_ffi;
extern crate mint_token;

use contract_ffi::contract_api::{self, Error};
use contract_ffi::unwrap_or_revert::UnwrapOrRevert;

const MINT_FUNCTION_NAME: &str = "mint_ext";

#[no_mangle]
pub extern "C" fn mint_ext() {
    mint_token::delegate();
}

#[no_mangle]
pub extern "C" fn call() {
    let uref = contract_api::store_function(MINT_FUNCTION_NAME, Default::default())
        .into_turef()
        .unwrap_or_revert_with(Error::UnexpectedContractPointerVariant)
        .into();

    contract_api::ret(&uref, &vec![uref]);
}