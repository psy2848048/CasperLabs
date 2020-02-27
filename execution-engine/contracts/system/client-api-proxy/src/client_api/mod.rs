mod error;

use alloc::string::String;

use contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::{PublicKey, PurseId},
    ApiError, U512,
};

use error::Error;

mod method_names {
    pub mod proxy {
        use super::internal;
        pub const BOND: &str = internal::BOND;
        pub const UNBOND: &str = internal::UNBOND;
        pub const STANDARD_PAYMENT: &str = "standard_payment";
        pub const TRANSFER_TO_ACCOUNT: &str = "transfer_to_account";
    }
    pub mod internal {
        pub const BOND: &str = "bond";
        pub const UNBOND: &str = "unbond";
        pub const GET_PAYMENT_PURSE: &str = "get_payment_purse";
    }
}

pub enum Api {
    Bond(u64),
    Unbond(Option<u64>),
    StandardPayment(u32),
    TransferToAccount(PublicKey, u64),
}

impl Api {
    pub fn from_args() -> Self {
        let method_name: String = runtime::get_arg(0)
            .unwrap_or_revert_with(ApiError::MissingArgument)
            .unwrap_or_revert_with(ApiError::InvalidArgument);

        match method_name.as_str() {
            method_names::proxy::BOND => {
                let amount: u64 = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Bond(amount)
            }
            method_names::proxy::UNBOND => {
                let amount: Option<u64> = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Unbond(amount)
            }
            method_names::proxy::STANDARD_PAYMENT => {
                let amount: u32 = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::StandardPayment(amount)
            }
            method_names::proxy::TRANSFER_TO_ACCOUNT => {
                let public_key: PublicKey = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let transfer_amount: u64 = runtime::get_arg(2)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);

                Api::TransferToAccount(public_key, transfer_amount)
            }
            _ => runtime::revert(Error::UnknownProxyApi),
        }
    }

    pub fn invoke(&self) {
        match self {
            Self::Bond(amount) => {
                let pos_ref = system::get_proof_of_stake();
                let amount: U512 = (*amount).into();

                let source_purse = account::get_main_purse();
                let bonding_purse = system::create_purse();

                system::transfer_from_purse_to_purse(source_purse, bonding_purse, amount)
                    .unwrap_or_revert();

                runtime::call_contract(
                    pos_ref,
                    (method_names::internal::BOND, amount, bonding_purse),
                )
            }
            Self::Unbond(amount) => {
                let pos_ref = system::get_proof_of_stake();
                let amount: Option<U512> = amount.map(Into::into);
                runtime::call_contract(pos_ref, (method_names::internal::UNBOND, amount))
            }
            Self::StandardPayment(amount) => {
                let pos_ref = system::get_proof_of_stake();
                let main_purse = account::get_main_purse();
                let payment_purse: PurseId =
                    runtime::call_contract(pos_ref, (method_names::internal::GET_PAYMENT_PURSE,));
                system::transfer_from_purse_to_purse(main_purse, payment_purse, (*amount).into())
                    .unwrap_or_revert();
            }
            Self::TransferToAccount(public_key, amount) => {
                system::transfer_to_account(*public_key, (*amount).into()).unwrap_or_revert();
            }
        }
    }
}
