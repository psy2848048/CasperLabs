mod keys;

use alloc::vec::Vec;

use contract::contract_api::storage;
use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

use super::requests::{ClaimRequest, RedelegateRequest, UnbondRequest, UndelegateRequest};

use crate::duration_queue::DurationQueue;

pub fn read_unbond_requests() -> DurationQueue<UnbondRequest> {
    storage::read_local(&keys::UNBOND_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_unbond_requests(queue: DurationQueue<UnbondRequest>) {
    storage::write_local(keys::UNBOND_REQUEST_QUEUE, queue);
}

pub fn read_undelegation_requests() -> DurationQueue<UndelegateRequest> {
    storage::read_local(&keys::UNDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_undelegation_requests(queue: DurationQueue<UndelegateRequest>) {
    storage::write_local(keys::UNDELEGATE_REQUEST_QUEUE, queue);
}

pub fn read_redelegation_requests() -> DurationQueue<RedelegateRequest> {
    storage::read_local(&keys::REDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_redelegation_requests(queue: DurationQueue<RedelegateRequest>) {
    storage::write_local(keys::REDELEGATE_REQUEST_QUEUE, queue);
}

pub fn read_claim_requests() -> Vec<ClaimRequest> {
    storage::read_local(&keys::CLAIM_REQUESTS)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_claim_requests(list: Vec<ClaimRequest>) {
    storage::write_local(keys::CLAIM_REQUESTS, list);
}

pub fn read_bonding_amount(user: PublicKey) -> U512 {
    let key = keys::bonding_amount_key(user);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_bonding_amount(user: PublicKey, amount: U512) {
    let key = keys::bonding_amount_key(user);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, amount);
}

pub fn read_vote(voter: PublicKey, dapp: Key) -> U512 {
    let key = keys::vote_key(voter, dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voting_amount(voter: PublicKey) -> U512 {
    let key = keys::voting_amount_key(voter);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voted_amount(dapp: Key) -> U512 {
    let key = keys::voted_amount_key(dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn vote(voter: PublicKey, dapp: Key, amount: U512) -> Result<()> {
    // update voting amount (voter, amount)
    let voting_amount_key = keys::voting_amount_key(voter);
    let voting_amount: U512 = storage::read_local(&voting_amount_key)
        .unwrap_or_default()
        .unwrap_or_default();

    // validate amount
    {
        if amount.is_zero() {
            // TODO: change to Error::VoteTooSmall
            return Err(Error::BondTooSmall);
        }

        let bonding_amount = read_bonding_amount(voter);

        if voting_amount > bonding_amount {
            // TODO: Internal Error
            return Err(Error::VoteTooLarge);
        }
        if amount > bonding_amount - voting_amount {
            return Err(Error::VoteTooLarge);
        }
    }
    storage::write_local(voting_amount_key, voting_amount + amount);

    // update vote ((voter, dapp), amount)
    let key = keys::vote_key(voter, dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    // update voted amount (dapp, amount)
    let key = keys::voted_amount_key(dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    Ok(())
}

pub fn unvote(voter: PublicKey, dapp: Key, amount: U512) -> Result<()> {
    // update vote ((voter, dapp), amount)
    let vote_key = keys::vote_key(voter, dapp);
    let vote_amount = storage::read_local(&vote_key)
        .unwrap_or_default()
        .unwrap_or_default();
    if amount > vote_amount {
        return Err(Error::UnvoteTooLarge);
    }
    storage::write_local(vote_key, vote_amount - amount);

    // update voting amount (voter, amount)
    let key = keys::voting_amount_key(voter);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);

    // update voted amount (dapp, amount)
    let key = keys::voted_amount_key(dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);
    Ok(())
}
