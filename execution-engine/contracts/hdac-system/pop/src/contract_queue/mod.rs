mod claim_request_list;
mod request_queue;
mod requests;

use contract::contract_api::storage;
use proof_of_stake::{self, QueueProvider};

use super::constants::local_keys;

use claim_request_list::ClaimRequestList;
use request_queue::{RequestKey, RequestQueue};
pub use requests::{ClaimRequest, DelegateRequestKey, RedelegateRequestKey, UndelegateRequestKey};

pub struct ContractQueue;

impl ContractQueue {
    pub fn read_requests<T: RequestKey + Default>(key: u8) -> RequestQueue<T> {
        storage::read_local(&key)
            .unwrap_or_default()
            .unwrap_or_default()
    }
    pub fn write_requests<T: RequestKey + Default>(key: u8, queue: RequestQueue<T>) {
        storage::write_local(key, queue);
    }

    pub fn read_claim_requests() -> ClaimRequestList {
        storage::read_local(&local_keys::CLAIM_REQUESTS)
            .unwrap_or_default()
            .unwrap_or_default()
    }
    pub fn write_claim_requests(list: ClaimRequestList) {
        storage::write_local(local_keys::CLAIM_REQUESTS, list);
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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use types::{account::PublicKey, system_contract_errors::pos::Error, BlockTime, U512};

    use super::{DelegateRequestKey, RequestQueue, UndelegateRequestKey};
    use crate::contract_queue::request_queue::RequestQueueEntry;

    const KEY1: [u8; 32] = [1; 32];
    const KEY2: [u8; 32] = [2; 32];
    const KEY3: [u8; 32] = [3; 32];
    const KEY4: [u8; 32] = [4; 32];

    #[test]
    fn test_request_queue_push() {
        let delegator = PublicKey::new(KEY1);
        let validator_1 = PublicKey::new(KEY2);
        let validator_2 = PublicKey::new(KEY3);
        let validator_3 = PublicKey::new(KEY4);

        let mut queue: RequestQueue<DelegateRequestKey> = Default::default();
        assert_eq!(
            Ok(()),
            queue.push(
                DelegateRequestKey::new(delegator, validator_1),
                U512::from(5),
                BlockTime::new(100)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                DelegateRequestKey::new(delegator, validator_2),
                U512::from(5),
                BlockTime::new(101)
            )
        );
        assert_eq!(
            Err(Error::MultipleRequests),
            queue.push(
                DelegateRequestKey::new(delegator, validator_1),
                U512::from(6),
                BlockTime::new(102)
            )
        );
        assert_eq!(
            Err(Error::TimeWentBackwards),
            queue.push(
                DelegateRequestKey::new(delegator, validator_3),
                U512::from(5),
                BlockTime::new(100)
            )
        );
    }

    #[test]
    fn test_request_queue_pop_due() {
        let delegator = PublicKey::new(KEY1);
        let validator_1 = PublicKey::new(KEY2);
        let validator_2 = PublicKey::new(KEY3);
        let validator_3 = PublicKey::new(KEY4);

        let mut queue: RequestQueue<UndelegateRequestKey> = Default::default();
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequestKey::new(delegator, validator_1),
                U512::from(5),
                BlockTime::new(100)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequestKey::new(delegator, validator_2),
                U512::from(5),
                BlockTime::new(101)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequestKey::new(delegator, validator_3),
                U512::from(5),
                BlockTime::new(102)
            )
        );
        assert_eq!(
            vec![
                RequestQueueEntry::new(
                    UndelegateRequestKey::new(delegator, validator_1),
                    U512::from(5),
                    BlockTime::new(100)
                ),
                RequestQueueEntry::new(
                    UndelegateRequestKey::new(delegator, validator_2),
                    U512::from(5),
                    BlockTime::new(101)
                ),
            ],
            queue.pop_due(BlockTime::new(101))
        );
        assert_eq!(
            vec![RequestQueueEntry::new(
                UndelegateRequestKey::new(delegator, validator_3),
                U512::from(5),
                BlockTime::new(102)
            ),],
            queue.pop_due(BlockTime::new(105))
        );
    }
}
