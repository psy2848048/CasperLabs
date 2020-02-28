//! Types used to allow creation of Wasm contracts and tests for use on the CasperLabs Platform.
//!
//! # `no_std`
//!
//! By default, the library is `no_std`, however you can enable full `std` functionality by enabling
//! the crate's `std` feature.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(specialization, try_reserve)]
#![doc(html_root_url = "https://docs.rs/casperlabs-types/0.2.0")]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/CasperLabs/CasperLabs/dev/images/CasperLabs_Logo_Favicon_RGB_50px.png",
    html_logo_url = "https://raw.githubusercontent.com/CasperLabs/CasperLabs/dev/images/CasperLabs_Logo_Symbol_RGB.png",
    test(attr(forbid(warnings)))
)]
#![warn(missing_docs)]

extern crate alloc;
#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

mod access_rights;
pub mod account;
pub mod api_error;
mod block_time;
pub mod bytesrepr;
mod cl_type;
mod cl_value;
mod contract_ref;
#[cfg(any(feature = "gens", test))]
pub mod gens;
mod key;
mod phase;
mod protocol_version;
mod semver;
pub mod system_contract_errors;
mod system_contract_type;
mod transfer_result;
mod uint;
mod uref;

pub use crate::uint::{UIntParseError, U128, U256, U512};
pub use access_rights::{AccessRights, ACCESS_RIGHTS_SERIALIZED_LENGTH};
#[doc(inline)]
pub use api_error::ApiError;
pub use block_time::{BlockTime, BLOCKTIME_SERIALIZED_LENGTH};
pub use cl_type::{named_key_type, CLType, CLTyped};
pub use cl_value::{CLTypeMismatch, CLValue, CLValueError};
pub use contract_ref::ContractRef;
#[doc(inline)]
pub use key::{Key, KEY_ACCOUNT_LENGTH, KEY_HASH_LENGTH, KEY_LOCAL_LENGTH, KEY_LOCAL_SEED_LENGTH};
pub use phase::{Phase, PHASE_SERIALIZED_LENGTH};
pub use protocol_version::{ProtocolVersion, VersionCheckResult};
pub use semver::SemVer;
pub use system_contract_type::SystemContractType;
pub use transfer_result::{TransferResult, TransferredTo};
pub use uref::{URef, UREF_ADDR_LENGTH, UREF_SERIALIZED_LENGTH};