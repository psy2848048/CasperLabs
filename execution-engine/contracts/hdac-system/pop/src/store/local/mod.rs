mod keys;

use alloc::vec::Vec;

use contract::contract_api::storage;
use types::{account::PublicKey, Key, U512};

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

pub fn write_vote(voter: PublicKey, dapp: Key, amount: U512) {
    let key = keys::vote_key(voter, dapp);
    storage::write_local(key, amount);
}

pub fn write_voting_amount(voter: PublicKey, amount: U512) {
    let key = keys::voting_amount_key(voter);
    storage::write_local(key, amount);
}

pub fn write_voted_amount(dapp: Key, amount: U512) {
    let key = keys::voted_amount_key(dapp);
    storage::write_local(key, amount);
}
