mod keys;

use contract::contract_api::storage;
use types::{account::PublicKey, Key, U512};

use super::requests::{RedelegateRequest, UnbondRequest, UndelegateRequest};

use crate::duration_queue::DurationQueue;

pub fn read_total_mint_supply() -> U512 {
    storage::read_local(&keys::TOTAL_MINT_SUPPLY)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_total_mint_supply(amount: U512) {
    storage::write_local(keys::TOTAL_MINT_SUPPLY, amount);
}

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

pub fn read_last_distributed_block() -> u64 {
    storage::read_local(&keys::LAST_DISTRIBUTED_BLOCK_HEIGHT)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_last_distributed_block(height: u64) {
    storage::write_local(keys::LAST_DISTRIBUTED_BLOCK_HEIGHT, height);
}

pub fn read_bonding_amount(user: &PublicKey) -> U512 {
    let key = keys::bonding_amount_key(user);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_bonding_amount(user: &PublicKey, amount: U512) {
    let key = keys::bonding_amount_key(user);
    storage::write_local(key, amount);
}

pub fn read_vote(voter: &PublicKey, dapp: &Key) -> U512 {
    let key = keys::vote_key(voter, dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voting_amount(voter: &PublicKey) -> U512 {
    let key = keys::voting_amount_key(voter);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voted_amount(dapp: &Key) -> U512 {
    let key = keys::voted_amount_key(dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_vote(voter: &PublicKey, dapp: &Key, amount: U512) {
    let key = keys::vote_key(voter, dapp);
    storage::write_local(key, amount);
}

pub fn write_voting_amount(voter: &PublicKey, amount: U512) {
    let key = keys::voting_amount_key(voter);
    storage::write_local(key, amount);
}

pub fn write_voted_amount(dapp: &Key, amount: U512) {
    let key = keys::voted_amount_key(dapp);
    storage::write_local(key, amount);
}

pub fn read_commission_amount(validator: &PublicKey) -> U512 {
    let key = keys::commission_amount_key(validator);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_commission_amount(validator: &PublicKey, amount: U512) {
    let key = keys::commission_amount_key(validator);
    storage::write_local(key, amount);
}

pub fn read_reward_amount(user: &PublicKey) -> U512 {
    let key = keys::reward_amount_key(user);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_reward_amount(user: &PublicKey, amount: U512) {
    let key = keys::reward_amount_key(user);
    storage::write_local(key, amount);
}
