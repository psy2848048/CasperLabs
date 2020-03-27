mod error;

use alloc::string::String;

use contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::{PublicKey, PurseId},
    ApiError, Key, U512,
};

use error::Error;

mod method_names {
    pub mod proxy {
        use super::pos;
        pub const BOND: &str = pos::BOND;
        pub const UNBOND: &str = pos::UNBOND;
        pub const STANDARD_PAYMENT: &str = "standard_payment";
        pub const TRANSFER_TO_ACCOUNT: &str = "transfer_to_account";
        pub const DELEGATE: &str = pos::DELEGATE;
        pub const UNDELEGATE: &str = pos::UNDELEGATE;
        pub const REDELEGATE: &str = pos::REDELEGATE;
        pub const VOTE: &str = pos::VOTE;
        pub const UNVOTE: &str = pos::UNVOTE;
        pub const DISTRIBUTE: &str = pos::DISTRIBUTE;
        pub const CLAIM_COMMISSION: &str = pos::CLAIM_COMMISSION;
        pub const CLAIM_REWARD: &str = pos::CLAIM_REWARD;
        pub const WRITE_GENESIS_TOTAL_SUPPLY: &str = pos::WRITE_GENESIS_TOTAL_SUPPLY;
    }
    pub mod pos {
        pub const BOND: &str = "bond";
        pub const UNBOND: &str = "unbond";
        pub const GET_PAYMENT_PURSE: &str = "get_payment_purse";
        pub const DELEGATE: &str = "delegate";
        pub const UNDELEGATE: &str = "undelegate";
        pub const REDELEGATE: &str = "redelegate";
        pub const VOTE: &str = "vote";
        pub const UNVOTE: &str = "unvote";
        pub const DISTRIBUTE: &str = "distribute";
        pub const CLAIM_COMMISSION: &str = "claim_commission";
        pub const CLAIM_REWARD: &str = "claim_reward";
        pub const WRITE_GENESIS_TOTAL_SUPPLY: &str = "write_genesis_total_supply";
    }
}

pub enum Api {
    Bond(U512),
    Unbond(Option<U512>),
    StandardPayment(U512),
    TransferToAccount(PublicKey, U512),
    Delegate(PublicKey, U512),
    Undelegate(PublicKey, Option<U512>),
    Redelegate(PublicKey, PublicKey, U512),
    Vote(Key, U512),
    Unvote(Key, Option<U512>),
    Distribute(),
    ClaimCommission(),
    ClaimReward(),
    WriteGenesisTotalSupply(U512),
}

impl Api {
    pub fn from_args() -> Self {
        let method_name: String = runtime::get_arg(0)
            .unwrap_or_revert_with(ApiError::MissingArgument)
            .unwrap_or_revert_with(ApiError::InvalidArgument);

        match method_name.as_str() {
            method_names::proxy::BOND => {
                let amount: U512 = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Bond(amount)
            }
            method_names::proxy::UNBOND => {
                let amount: Option<U512> = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Unbond(amount)
            }
            method_names::proxy::STANDARD_PAYMENT => {
                let amount: U512 = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::StandardPayment(amount)
            }
            method_names::proxy::TRANSFER_TO_ACCOUNT => {
                let public_key: PublicKey = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let transfer_amount: U512 = runtime::get_arg(2)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);

                Api::TransferToAccount(public_key, transfer_amount)
            }
            method_names::proxy::DELEGATE => {
                let validator: PublicKey = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let amount: U512 = runtime::get_arg(2)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Delegate(validator, amount)
            }
            method_names::proxy::UNDELEGATE => {
                let validator: PublicKey = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let amount: Option<U512> = runtime::get_arg(2)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Undelegate(validator, amount)
            }
            method_names::proxy::REDELEGATE => {
                let src_validator: PublicKey = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let dest_validator: PublicKey = runtime::get_arg(2)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let amount: U512 = runtime::get_arg(3)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Redelegate(src_validator, dest_validator, amount)
            }
            method_names::proxy::VOTE => {
                let dapp: Key = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let amount: U512 = runtime::get_arg(2)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Vote(dapp, amount)
            }
            method_names::proxy::UNVOTE => {
                let dapp: Key = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                let amount: Option<U512> = runtime::get_arg(2)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::Unvote(dapp, amount)
            }
            method_names::proxy::DISTRIBUTE => {
                Api::Distribute()
            }
            method_names::proxy::CLAIM_COMMISSION => {
                Api::ClaimCommission()
            }
            method_names::proxy::CLAIM_REWARD => {
                Api::ClaimReward()
            }
            method_names::proxy::WRITE_GENESIS_TOTAL_SUPPLY => {
                let amount: U512 = runtime::get_arg(1)
                    .unwrap_or_revert_with(ApiError::MissingArgument)
                    .unwrap_or_revert_with(ApiError::InvalidArgument);
                Api::WriteGenesisTotalSupply(amount)
            }
            _ => runtime::revert(Error::UnknownProxyApi),
        }
    }

    pub fn invoke(&self) {
        match self {
            Self::Bond(amount) => {
                let pos_ref = system::get_proof_of_stake();

                let source_purse = account::get_main_purse();
                let bonding_purse = system::create_purse();

                system::transfer_from_purse_to_purse(source_purse, bonding_purse, *amount)
                    .unwrap_or_revert();

                runtime::call_contract(pos_ref, (method_names::pos::BOND, *amount, bonding_purse))
            }
            Self::Unbond(amount) => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(pos_ref, (method_names::pos::UNBOND, *amount))
            }
            Self::StandardPayment(amount) => {
                let pos_ref = system::get_proof_of_stake();
                let main_purse = account::get_main_purse();
                let payment_purse: PurseId =
                    runtime::call_contract(pos_ref, (method_names::pos::GET_PAYMENT_PURSE,));
                system::transfer_from_purse_to_purse(main_purse, payment_purse, *amount)
                    .unwrap_or_revert();
            }
            Self::TransferToAccount(public_key, amount) => {
                system::transfer_to_account(*public_key, *amount).unwrap_or_revert();
            }
            Self::Delegate(validator, amount) => {
                let pos_ref = system::get_proof_of_stake();

                let source_purse = account::get_main_purse();
                let bonding_purse = system::create_purse();

                system::transfer_from_purse_to_purse(source_purse, bonding_purse, *amount)
                    .unwrap_or_revert();

                runtime::call_contract(
                    pos_ref,
                    (
                        method_names::pos::DELEGATE,
                        *validator,
                        *amount,
                        bonding_purse,
                    ),
                )
            }
            Self::Undelegate(validator, amount) => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(
                    pos_ref,
                    (method_names::pos::UNDELEGATE, *validator, *amount),
                )
            }
            Self::Redelegate(src, dest, amount) => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(
                    pos_ref,
                    (method_names::pos::REDELEGATE, *src, *dest, *amount),
                )
            }
            Self::Vote(dapp, amount) => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(pos_ref, (method_names::pos::VOTE, *dapp, *amount))
            }
            Self::Unvote(dapp, amount) => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(pos_ref, (method_names::pos::UNVOTE, *dapp, *amount))
            }
            Self::Distribute() => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(pos_ref, (method_names::pos::DISTRIBUTE, ))
            }
            Self::ClaimCommission() => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(pos_ref, (method_names::pos::CLAIM_COMMISSION, ))
            }
            Self::ClaimReward() => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(pos_ref, (method_names::pos::CLAIM_REWARD, ))
            }
            Self::WriteGenesisTotalSupply(amount) => {
                let pos_ref = system::get_proof_of_stake();
                runtime::call_contract(pos_ref, (method_names::pos::WRITE_GENESIS_TOTAL_SUPPLY, *amount))
            }
        }
    }
}
