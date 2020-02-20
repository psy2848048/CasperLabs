#![no_std]

extern crate alloc;

mod client_api;

use contract::contract_api::{runtime, storage};

use client_api::Api;

const CLIENT_API_PROXY_NAME: &str = "client_api_proxy";

#[no_mangle]
pub extern "C" fn client_api_proxy() {
    Api::from_args().invoke();
}

pub fn deploy_client_api_proxy() {
    let contract_hash = storage::store_function_at_hash(CLIENT_API_PROXY_NAME, Default::default());
    runtime::put_key(CLIENT_API_PROXY_NAME, contract_hash.into());
}

#[cfg(not(feature = "lib"))]
#[no_mangle]
pub extern "C" fn call() {
    deploy_client_api_proxy();
}
