use alloc::{boxed::Box, vec::Vec};
use core::result;

use types::{
    bytesrepr::{self, FromBytes, ToBytes},
    system_contract_errors::pos::{Error, Result},
    BlockTime, CLType, CLTyped, U512,
};

#[derive(Default, PartialEq)]
pub struct RequestQueue<T: RequestKey>(pub Vec<RequestQueueEntry<T>>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestQueueEntry<T: RequestKey> {
    request_key: T,
    amount: U512,
    timestamp: BlockTime,
}

impl<T: RequestKey> RequestQueueEntry<T> {
    pub fn new(request_key: T, amount: U512, timestamp: BlockTime) -> Self {
        RequestQueueEntry::<T> {
            request_key,
            amount,
            timestamp,
        }
    }
}

pub trait RequestKey: Clone + Copy + PartialEq + FromBytes + ToBytes + CLTyped {}

impl<T: RequestKey> RequestQueue<T> {
    pub fn push(&mut self, request_key: T, amount: U512, timestamp: BlockTime) -> Result<()> {
        if self.0.iter().any(|entry| entry.request_key == request_key) {
            return Err(Error::MultipleRequests);
        }
        if let Some(entry) = self.0.last() {
            if entry.timestamp > timestamp {
                return Err(Error::TimeWentBackwards);
            }
        }
        self.0.push(RequestQueueEntry {
            request_key,
            amount,
            timestamp,
        });
        Ok(())
    }
    pub fn pop_due(&mut self, timestamp: BlockTime) -> Vec<RequestQueueEntry<T>> {
        let (older_than, rest) = self
            .0
            .iter()
            .partition(|entry| entry.timestamp <= timestamp);
        self.0 = rest;
        older_than
    }
}

impl<T: RequestKey> FromBytes for RequestQueue<T> {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (len, mut bytes) = u64::from_bytes(bytes)?;
        let mut queue = Vec::new();
        for _ in 0..len {
            let (entry, rest) = RequestQueueEntry::from_bytes(bytes)?;
            bytes = rest;
            queue.push(entry);
        }
        Ok((RequestQueue(queue), bytes))
    }
}

impl<T: RequestKey> ToBytes for RequestQueue<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let mut bytes = (self.0.len() as u64).to_bytes()?; // TODO: Allocate correct capacity.
        for entry in &self.0 {
            bytes.append(&mut entry.to_bytes()?);
        }
        Ok(bytes)
    }
}

impl<T: RequestKey> CLTyped for RequestQueue<T> {
    fn cl_type() -> CLType {
        CLType::List(Box::new(RequestQueueEntry::<T>::cl_type()))
    }
}

impl<T: RequestKey> FromBytes for RequestQueueEntry<T> {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (request_key, bytes) = T::from_bytes(bytes)?;
        let (amount, bytes) = U512::from_bytes(bytes)?;
        let (timestamp, bytes) = BlockTime::from_bytes(bytes)?;
        let entry = RequestQueueEntry {
            request_key,
            amount,
            timestamp,
        };
        Ok((entry, bytes))
    }
}

impl<T: RequestKey> ToBytes for RequestQueueEntry<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.request_key.to_bytes()?.into_iter())
            .chain(self.amount.to_bytes()?)
            .chain(self.timestamp.to_bytes()?)
            .collect())
    }
}

impl<T: RequestKey> CLTyped for RequestQueueEntry<T> {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
