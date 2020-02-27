#![no_std]

// Required to bring `#[panic_handler]` from `contract::handlers` into scope.
#[allow(unused_imports, clippy::single_component_path_imports)]
use contract;

#[no_mangle]
pub extern "C" fn call() {
    // This body intentionally left empty.
}
