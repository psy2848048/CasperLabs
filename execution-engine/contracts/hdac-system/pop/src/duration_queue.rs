use alloc::{boxed::Box, vec::Vec};
use core::result;

use types::{
    bytesrepr::{self, FromBytes, ToBytes, U64_SERIALIZED_LENGTH},
    system_contract_errors::pos::{Error, Result},
    BlockTime, CLType, CLTyped,
};

#[derive(Default)]
pub struct DurationQueue<T: DurationQueueItem>(pub Vec<DurationQueueEntry<T>>);

#[derive(Debug, Clone, Copy)]
pub struct DurationQueueEntry<T: DurationQueueItem> {
    pub item: T,
    pub timestamp: BlockTime,
}

pub trait DurationQueueItem: CLTyped + FromBytes + ToBytes + Copy + Clone {}

impl<T: DurationQueueItem> DurationQueue<T> {
    pub fn push(&mut self, item: T, timestamp: BlockTime) -> Result<()> {
        if let Some(entry) = self.0.last() {
            if entry.timestamp > timestamp {
                return Err(Error::TimeWentBackwards);
            }
        }
        self.0.push(DurationQueueEntry { item, timestamp });
        Ok(())
    }
    pub fn pop_due(&mut self, timestamp: BlockTime) -> Vec<DurationQueueEntry<T>> {
        let (older_than, rest) = self
            .0
            .iter()
            .partition(|entry| entry.timestamp <= timestamp);
        self.0 = rest;
        older_than
    }
}

impl<T: DurationQueueItem> FromBytes for DurationQueue<T> {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (len, mut bytes) = u64::from_bytes(bytes)?;
        let mut queue = Vec::new();
        for _ in 0..len {
            let (entry, rest) = DurationQueueEntry::from_bytes(bytes)?;
            bytes = rest;
            queue.push(entry);
        }
        Ok((DurationQueue(queue), bytes))
    }
}

impl<T: DurationQueueItem> ToBytes for DurationQueue<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let mut bytes = (self.0.len() as u64).to_bytes()?; // TODO: Allocate correct capacity.
        for entry in &self.0 {
            bytes.append(&mut entry.to_bytes()?);
        }
        Ok(bytes)
    }
    fn serialized_length(&self) -> usize {
        U64_SERIALIZED_LENGTH + self.0.iter().map(ToBytes::serialized_length).sum::<usize>()
    }
}

impl<T: DurationQueueItem> CLTyped for DurationQueue<T> {
    fn cl_type() -> CLType {
        CLType::List(Box::new(DurationQueueEntry::<T>::cl_type()))
    }
}

impl<T: DurationQueueItem> FromBytes for DurationQueueEntry<T> {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (item, bytes) = T::from_bytes(bytes)?;
        let (timestamp, bytes) = BlockTime::from_bytes(bytes)?;
        let entry = DurationQueueEntry { item, timestamp };
        Ok((entry, bytes))
    }
}

impl<T: DurationQueueItem> ToBytes for DurationQueueEntry<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.item.to_bytes()?.into_iter())
            .chain(self.timestamp.to_bytes()?)
            .collect())
    }
    fn serialized_length(&self) -> usize {
        self.item.serialized_length() + self.timestamp.serialized_length()
    }
}

impl<T: DurationQueueItem> CLTyped for DurationQueueEntry<T> {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use types::{account::PublicKey, system_contract_errors::pos::Error, BlockTime, U512};

    use crate::local_store::UndelegateRequest;

    use super::{DurationQueue, DurationQueueEntry};

    const KEY1: [u8; 32] = [1; 32];
    const KEY2: [u8; 32] = [2; 32];
    const KEY3: [u8; 32] = [3; 32];
    const KEY4: [u8; 32] = [4; 32];

    #[test]
    fn test_request_queue_push() {
        let delegator = PublicKey::ed25519_from(KEY1);
        let validator_1 = PublicKey::ed25519_from(KEY2);
        let validator_2 = PublicKey::ed25519_from(KEY3);
        let validator_3 = PublicKey::ed25519_from(KEY4);

        let mut queue: DurationQueue<UndelegateRequest> = Default::default();
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequest {
                    delegator: delegator,
                    validator: validator_1,
                    maybe_amount: Some(U512::from(5))
                },
                BlockTime::new(100)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequest {
                    delegator: delegator,
                    validator: validator_2,
                    maybe_amount: Some(U512::from(5))
                },
                BlockTime::new(101)
            )
        );
        assert_eq!(
            Err(Error::MultipleRequests),
            queue.push(
                UndelegateRequest {
                    delegator: delegator,
                    validator: validator_1,
                    maybe_amount: Some(U512::from(6))
                },
                BlockTime::new(102)
            )
        );
        assert_eq!(
            Err(Error::TimeWentBackwards),
            queue.push(
                UndelegateRequest {
                    delegator: delegator,
                    validator: validator_3,
                    maybe_amount: Some(U512::from(5))
                },
                BlockTime::new(100)
            )
        );
    }

    #[test]
    fn test_request_queue_pop_due() {
        let delegator = PublicKey::ed25519_from(KEY1);
        let validator_1 = PublicKey::ed25519_from(KEY2);
        let validator_2 = PublicKey::ed25519_from(KEY3);
        let validator_3 = PublicKey::ed25519_from(KEY4);

        let mut queue: DurationQueue<UndelegateRequest> = Default::default();
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequest {
                    delegator: delegator,
                    validator: validator_1,
                    maybe_amount: Some(U512::from(5))
                },
                BlockTime::new(100)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequest {
                    delegator: delegator,
                    validator: validator_2,
                    maybe_amount: Some(U512::from(5))
                },
                BlockTime::new(101)
            )
        );
        assert_eq!(
            Ok(()),
            queue.push(
                UndelegateRequest {
                    delegator: delegator,
                    validator: validator_3,
                    maybe_amount: Some(U512::from(5))
                },
                BlockTime::new(102)
            )
        );
        assert_eq!(
            vec![
                DurationQueueEntry {
                    item: UndelegateRequest {
                        delegator: delegator,
                        validator: validator_1,
                        maybe_amount: Some(U512::from(5))
                    },
                    timestamp: BlockTime::new(100)
                },
                DurationQueueEntry {
                    item: UndelegateRequest {
                        delegator: delegator,
                        validator: validator_2,
                        maybe_amount: Some(U512::from(5))
                    },
                    timestamp: BlockTime::new(101)
                },
            ],
            queue.pop_due(BlockTime::new(101))
        );
        assert_eq!(
            vec![DurationQueueEntry {
                item: UndelegateRequest {
                    delegator: delegator,
                    validator: validator_3,
                    maybe_amount: Some(U512::from(5))
                },
                timestamp: BlockTime::new(102)
            },],
            queue.pop_due(BlockTime::new(105))
        );
    }
}
