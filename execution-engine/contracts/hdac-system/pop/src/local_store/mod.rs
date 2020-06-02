mod local_keys;
mod requests;

use alloc::vec::Vec;

use contract::contract_api::storage;
use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

pub use requests::{ClaimRequest, RedelegateRequest, UnbondRequest, UndelegateRequest};

use crate::duration_queue::DurationQueue;

pub fn read_unbond_requests() -> DurationQueue<UnbondRequest> {
    storage::read_local(&local_keys::UNBOND_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_unbond_requests(queue: DurationQueue<UnbondRequest>) {
    storage::write_local(local_keys::UNBOND_REQUEST_QUEUE, queue);
}

pub fn read_undelegation_requests() -> DurationQueue<UndelegateRequest> {
    storage::read_local(&local_keys::UNDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_undelegation_requests(queue: DurationQueue<UndelegateRequest>) {
    storage::write_local(local_keys::UNDELEGATE_REQUEST_QUEUE, queue);
}

pub fn read_redelegation_requests() -> DurationQueue<RedelegateRequest> {
    storage::read_local(&local_keys::REDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_redelegation_requests(queue: DurationQueue<RedelegateRequest>) {
    storage::write_local(local_keys::REDELEGATE_REQUEST_QUEUE, queue);
}

pub fn read_claim_requests() -> Vec<ClaimRequest> {
    storage::read_local(&local_keys::CLAIM_REQUESTS)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_claim_requests(list: Vec<ClaimRequest>) {
    storage::write_local(local_keys::CLAIM_REQUESTS, list);
}

pub fn read_bonding_amount(user: PublicKey) -> U512 {
    let key = local_keys::staking_amount_key(user);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn bond(user: PublicKey, amount: U512) {
    let key = local_keys::staking_amount_key(user);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);
}

pub fn unbond(user: PublicKey, amount: U512) -> Result<()> {
    let key = local_keys::staking_amount_key(user);
    let current_amount = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    if amount > current_amount {
        return Err(Error::UnbondTooLarge);
    }
    storage::write_local(key, current_amount - amount);
    Ok(())
}

pub fn read_delegation(delegator: PublicKey, validator: PublicKey) -> U512 {
    let key = local_keys::delegation_key(delegator, validator);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_delegating_amount(delegator: PublicKey) -> U512 {
    let key = local_keys::delegating_amount_key(delegator);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_delegated_amount(validator: PublicKey) -> U512 {
    let key = local_keys::delegated_amount_key(validator);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn delegate(delegator: PublicKey, validator: PublicKey, amount: U512) {
    // update delegation ((delegator, validator), amount)
    let key = local_keys::delegation_key(delegator, validator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    // update delegating amount (delegator, amount)
    let key = local_keys::delegating_amount_key(delegator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    // update delegated amount (validator, amount)
    let key = local_keys::delegated_amount_key(validator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);
}

pub fn undelegate(delegator: PublicKey, validator: PublicKey, amount: U512) -> Result<()> {
    // update delegation ((delegator, validator), amount)
    let key = local_keys::delegation_key(delegator, validator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    if amount > current_amount {
        return Err(Error::UndelegateTooLarge);
    }
    storage::write_local(key, current_amount - amount);

    // update delegating amount (delegator, amount)
    let key = local_keys::delegating_amount_key(delegator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);

    // update delegated amount (validator, amount)
    let key = local_keys::delegated_amount_key(validator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);
    Ok(())
}

pub fn read_vote(voter: PublicKey, dapp: Key) -> U512 {
    let key = local_keys::vote_key(voter, dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voting_amount(voter: PublicKey) -> U512 {
    let key = local_keys::voting_amount_key(voter);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voted_amount(dapp: Key) -> U512 {
    let key = local_keys::voted_amount_key(dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn vote(voter: PublicKey, dapp: Key, amount: U512) {
    // update vote ((voter, dapp), amount)
    let key = local_keys::vote_key(voter, dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    // update voting amount (voter, amount)
    let key = local_keys::voting_amount_key(voter);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    // update voted amount (dapp, amount)
    let key = local_keys::voted_amount_key(dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);
}

pub fn unvote(voter: PublicKey, dapp: Key, amount: U512) -> Result<()> {
    // update vote ((voter, dapp), amount)
    let key = local_keys::vote_key(voter, dapp);
    let current_amount = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    if amount > current_amount {
        return Err(Error::UnvoteTooLarge);
    }
    storage::write_local(key, current_amount - amount);

    // update voting amount (voter, amount)
    let key = local_keys::voting_amount_key(voter);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);

    // update voted amount (dapp, amount)
    let key = local_keys::voted_amount_key(dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);
    Ok(())
}
