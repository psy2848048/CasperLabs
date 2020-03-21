mod request_queue;
mod requests;

use contract::contract_api::storage;
use proof_of_stake::{self, QueueProvider};

use request_queue::{Request, RequestQueue};
pub use requests::{RedelegateRequest, UndelegateRequest};

pub struct ContractQueue;

impl ContractQueue {
    pub fn read_requests<T: Request + Default>(key: [u8; 32]) -> RequestQueue<T> {
        storage::read_local(&key)
            .unwrap_or_default()
            .unwrap_or_default()
    }
    pub fn write_requests<T: Request + Default>(key: [u8; 32], queue: RequestQueue<T>) {
        storage::write_local(key, queue);
    }
}

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
