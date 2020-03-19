use contract::contract_api::storage;
use proof_of_stake::{self, QueueProvider};

use crate::constants::local_keys::UNDELEGATE_REQUEST_QUEUE;

pub struct ContractQueue;

impl ContractQueue {
    pub fn read_undelegate_requests() -> UndelegateRequestQueue {
        todo!()
        // storage::read_local(&UNDELEGATE_REQUEST_QUEUE)
        //     .unwrap_or_default()
        //     .unwrap_or_default()
    }

    pub fn read_redelegate_requests() -> RedelegateRequestQueue {
        todo!()
        // storage::read_local(&REDELEGATE_REQUEST_QUEUE)
        //     .unwrap_or_default()
        //     .unwrap_or_default()
    }

    pub fn write_undelegate_requests(queue: UndelegateRequestQueue) {
        // storage::write_local(UNDELEGATE_REQUEST_QUEUE, queue);
    }

    pub fn write_redelegate_requests(queue: RedelegateRequestQueue) {
        // storage::write_local(REDELEGATE_REQUEST_QUEUE, queue);
    }
}

#[derive(Clone, Default)]
pub struct UndelegateRequestQueue;
#[derive(Clone, Default)]
pub struct RedelegateRequestQueue;

// TODO: remove QueueProvider
// Currently, we are utilizing the default implemention of the Proof-of-Stake crate,
// so we need to add a dummy implemention to meet trait contraint.
impl QueueProvider for ContractQueue {
    fn read_bonding() -> proof_of_stake::Queue {
        unimplemented!()
    }

    fn read_unbonding() -> proof_of_stake::Queue {
        unimplemented!()
    }

    fn write_bonding(_: proof_of_stake::Queue) {
        unimplemented!()
    }

    fn write_unbonding(_: proof_of_stake::Queue) {
        unimplemented!()
    }
}
