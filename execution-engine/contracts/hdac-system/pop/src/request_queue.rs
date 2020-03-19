use alloc::{boxed::Box, vec::Vec};
use core::result;

use types::{
    bytesrepr::{self, FromBytes, ToBytes},
    system_contract_errors::pos::{Error, Result},
    BlockTime, CLType, CLTyped,
};

pub struct RequestQueue<T: Request>(pub Vec<RequestQueueEntry<T>>);

#[derive(Clone, Copy, Debug)]
pub struct RequestQueueEntry<T: Request> {
    pub request: T,
    pub timestamp: BlockTime,
}

pub trait Request: Clone + Copy + FromBytes + ToBytes + CLTyped {
    fn is_same(&self, rhs: &Self) -> bool;
}

impl<T: Request> RequestQueue<T> {
    pub fn push(&mut self, request: T, timestamp: BlockTime) -> Result<()> {
        if self.0.iter().any(|entry| entry.request.is_same(&request)) {
            return Err(Error::MultipleRequests);
        }
        if let Some(entry) = self.0.last() {
            if entry.timestamp > timestamp {
                return Err(Error::TimeWentBackwards);
            }
        }
        self.0.push(RequestQueueEntry { request, timestamp });
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

impl<T: Request> FromBytes for RequestQueue<T> {
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

impl<T: Request> ToBytes for RequestQueue<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let mut bytes = (self.0.len() as u64).to_bytes()?; // TODO: Allocate correct capacity.
        for entry in &self.0 {
            bytes.append(&mut entry.to_bytes()?);
        }
        Ok(bytes)
    }
}

impl<T: Request> CLTyped for RequestQueue<T> {
    fn cl_type() -> CLType {
        CLType::List(Box::new(RequestQueueEntry::<T>::cl_type()))
    }
}

impl<T: Request> FromBytes for RequestQueueEntry<T> {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (request, bytes) = T::from_bytes(bytes)?;
        let (timestamp, bytes) = BlockTime::from_bytes(bytes)?;
        let entry = RequestQueueEntry { request, timestamp };
        Ok((entry, bytes))
    }
}

impl<T: Request> ToBytes for RequestQueueEntry<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.request.to_bytes()?.into_iter())
            .chain(self.timestamp.to_bytes()?)
            .collect())
    }
}

impl<T: Request> CLTyped for RequestQueueEntry<T> {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
