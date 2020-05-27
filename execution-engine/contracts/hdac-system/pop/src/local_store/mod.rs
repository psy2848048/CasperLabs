mod requests;

use alloc::vec::Vec;

use contract::contract_api::storage;

pub use requests::{ClaimRequest, RedelegateRequest, UndelegateRequest};

use crate::{constants::local_keys, duration_queue::DurationQueue};

pub fn read_undelegation_requests() -> DurationQueue<UndelegateRequest> {
    storage::read_local(&local_keys::UNDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_redelegation_requests() -> DurationQueue<RedelegateRequest> {
    storage::read_local(&local_keys::REDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_undelegation_requests(queue: DurationQueue<UndelegateRequest>) {
    storage::write_local(local_keys::UNDELEGATE_REQUEST_QUEUE, queue);
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
