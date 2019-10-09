#![no_std]

extern crate contract_ffi;

use contract_ffi::contract_api::{self, Error};
use contract_ffi::execution::Phase;
use contract_ffi::unwrap_or_revert::UnwrapOrRevert;

#[no_mangle]
pub extern "C" fn call() {
    let known_phase: Phase = contract_api::get_arg(0)
        .unwrap_or_revert_with(Error::MissingArgument)
        .unwrap_or_revert_with(Error::InvalidArgument);
    let get_phase = contract_api::get_phase();
    assert_eq!(
        get_phase, known_phase,
        "get_phase did not return known_phase"
    );
}