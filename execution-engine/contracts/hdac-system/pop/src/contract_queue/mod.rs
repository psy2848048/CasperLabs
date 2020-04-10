mod request_queue;
mod requests;

use contract::contract_api::storage;
use proof_of_stake::{self, QueueProvider};

use request_queue::{ClaimQueue, RequestKey, RequestQueue};
pub use requests::{
    ClaimKeyType, ClaimRequestKey, DelegateRequestKey, RedelegateRequestKey, UndelegateRequestKey,
};

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

    pub fn read_claim_requests<T: RequestKey + Default>(key: u8) -> ClaimQueue<T> {
        storage::read_local(&key)
            .unwrap_or_default()
            .unwrap_or_default()
    }
    pub fn write_claim_requests<T: RequestKey + Default>(key: u8, queue: ClaimQueue<T>) {
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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use types::{account::PublicKey, system_contract_errors::pos::Error, BlockTime, U512};

    use super::{
        ClaimQueue, ClaimRequestKey, DelegateRequestKey, RequestQueue, UndelegateRequestKey,
    };
    use crate::contract_queue::{
        request_queue::{ClaimQueueEntry, RequestQueueEntry},
        requests::ClaimKeyType,
    };

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

    #[test]
    fn test_claim_queue_push() {
        let validator_1 = PublicKey::new(KEY2);
        let validator_2 = PublicKey::new(KEY3);
        let user_1 = PublicKey::new(KEY4);

        let mut queue: ClaimQueue<ClaimRequestKey> = Default::default();
        assert_eq!(
            Ok(()),
            queue.push(
                ClaimRequestKey::new(ClaimKeyType::Commission, validator_1),
                U512::from(5)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                ClaimRequestKey::new(ClaimKeyType::Commission, validator_2),
                U512::from(5)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                ClaimRequestKey::new(ClaimKeyType::Reward, user_1),
                U512::from(5)
            )
        );
        assert_eq!(
            Err(Error::MultipleRequests),
            queue.push(
                ClaimRequestKey::new(ClaimKeyType::Commission, validator_2),
                U512::from(5)
            )
        );
    }

    #[test]
    fn test_claim_queue_pop() {
        let validator_1 = PublicKey::new(KEY2);
        let validator_2 = PublicKey::new(KEY3);
        let user_1 = PublicKey::new(KEY4);

        let mut queue: ClaimQueue<ClaimRequestKey> = Default::default();
        assert_eq!(
            Ok(()),
            queue.push(
                ClaimRequestKey::new(ClaimKeyType::Commission, validator_1),
                U512::from(5)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                ClaimRequestKey::new(ClaimKeyType::Commission, validator_2),
                U512::from(5)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                ClaimRequestKey::new(ClaimKeyType::Reward, user_1),
                U512::from(5)
            )
        );
        assert_eq!(
            vec![
                ClaimQueueEntry::new(
                    ClaimRequestKey::new(ClaimKeyType::Commission, validator_1),
                    U512::from(5)
                ),
                ClaimQueueEntry::new(
                    ClaimRequestKey::new(ClaimKeyType::Reward, user_1),
                    U512::from(5)
                ),
            ],
            queue.pop(ClaimRequestKey::new(ClaimKeyType::Commission, validator_2))
        );
        assert_eq!(
            vec![ClaimQueueEntry::new(
                ClaimRequestKey::new(ClaimKeyType::Commission, validator_1),
                U512::from(5)
            ),],
            queue.pop(ClaimRequestKey::new(ClaimKeyType::Reward, user_1))
        );
    }
}
