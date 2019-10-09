#![no_std]

#[macro_use]
extern crate alloc;

extern crate contract_ffi;

use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;

use contract_ffi::contract_api::pointers::ContractPointer;
use contract_ffi::contract_api::{self, Error};
use contract_ffi::key::Key;
use contract_ffi::unwrap_or_revert::UnwrapOrRevert;
use contract_ffi::uref::URef;
use contract_ffi::value::U512;

#[no_mangle]
pub extern "C" fn do_nothing() {
    // Doesn't advance RNG of the runtime
    contract_api::ret(&String::from("Hello, world!"), &vec![])
}

#[no_mangle]
pub extern "C" fn do_something() {
    // Advances RNG of the runtime
    let test_string = String::from("Hello, world!");

    let test_uref = contract_api::new_turef(test_string).into();
    contract_api::ret(&test_uref, &vec![test_uref])
}

#[no_mangle]
pub extern "C" fn call() {
    let flag: String = contract_api::get_arg(0)
        .unwrap_or_revert_with(Error::MissingArgument)
        .unwrap_or_revert_with(Error::InvalidArgument);
    let do_nothing: ContractPointer =
        contract_api::store_function_at_hash("do_nothing", BTreeMap::new());
    let do_something: ContractPointer =
        contract_api::store_function_at_hash("do_something", BTreeMap::new());
    if flag == "pass1" {
        // Two calls should forward the internal RNG. This pass is a baseline.
        let uref1: URef = contract_api::new_turef(U512::from(0)).into();
        let uref2: URef = contract_api::new_turef(U512::from(1)).into();
        contract_api::put_key("uref1", &Key::URef(uref1));
        contract_api::put_key("uref2", &Key::URef(uref2));
    } else if flag == "pass2" {
        let uref1: URef = contract_api::new_turef(U512::from(0)).into();
        contract_api::put_key("uref1", &Key::URef(uref1));
        // do_nothing doesn't do anything. It SHOULD not forward the internal RNG.
        let result: String = contract_api::call_contract(do_nothing.clone(), &(), &vec![]);
        assert_eq!(result, "Hello, world!");
        let uref2: URef = contract_api::new_turef(U512::from(1)).into();
        contract_api::put_key("uref2", &Key::URef(uref2));
    } else if flag == "pass3" {
        let uref1: URef = contract_api::new_turef(U512::from(0)).into();
        contract_api::put_key("uref1", &Key::URef(uref1));
        // do_something returns a new uref, and it should forward the internal RNG.
        let uref2: URef = contract_api::call_contract(do_something.clone(), &(), &vec![]);
        contract_api::put_key("uref2", &Key::URef(uref2));
    }
}