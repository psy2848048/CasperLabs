use alloc::{boxed::Box, vec::Vec};
use core::result;

use types::{
    bytesrepr::{self, FromBytes, ToBytes},
    system_contract_errors::pos::{Error, Result},
    BlockTime, CLType, CLTyped, U512,
};

///////////////////
// Request queue
///////////////////
#[derive(Default, PartialEq)]
pub struct RequestQueue<T: RequestKey>(pub Vec<RequestQueueEntry<T>>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestQueueEntry<T: RequestKey> {
    pub request_key: T,
    pub amount: U512,
    pub timestamp: BlockTime,
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

///////////////////
// Claim queue
///////////////////
#[derive(Default, PartialEq)]
pub struct ClaimQueue<T: RequestKey>(pub Vec<ClaimQueueEntry<T>>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClaimQueueEntry<T: RequestKey> {
    pub request_key: T,
    pub amount: U512,
}

impl<T: RequestKey> ClaimQueueEntry<T> {
    pub fn new(request_key: T, amount: U512) -> Self {
        ClaimQueueEntry::<T> {
            request_key,
            amount,
        }
    }
}

impl<T: RequestKey> ClaimQueue<T> {
    pub fn push(&mut self, request_key: T, amount: U512) -> Result<()> {
        if self.0.iter().any(|entry| entry.request_key == request_key) {
            return Err(Error::MultipleRequests);
        }
        self.0.push(ClaimQueueEntry {
            request_key,
            amount,
        });
        Ok(())
    }

    pub fn pop(&mut self, request_key: T) -> Vec<ClaimQueueEntry<T>> {
        let idx = self.0.iter().position(|x| x.request_key == request_key).unwrap_or_default();
        self.0.remove(idx);

        let res = self.0.clone();

        res
    }
}

impl<T: RequestKey> FromBytes for ClaimQueue<T> {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (len, mut bytes) = u64::from_bytes(bytes)?;
        let mut queue = Vec::new();
        for _ in 0..len {
            let (entry, rest) = ClaimQueueEntry::from_bytes(bytes)?;
            bytes = rest;
            queue.push(entry);
        }
        Ok((ClaimQueue(queue), bytes))
    }
}

impl<T: RequestKey> ToBytes for ClaimQueue<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let mut bytes = (self.0.len() as u64).to_bytes()?; // TODO: Allocate correct capacity.
        for entry in &self.0 {
            bytes.append(&mut entry.to_bytes()?);
        }
        Ok(bytes)
    }
}

impl<T: RequestKey> CLTyped for ClaimQueue<T> {
    fn cl_type() -> CLType {
        CLType::List(Box::new(ClaimQueueEntry::<T>::cl_type()))
    }
}

impl<T: RequestKey> FromBytes for ClaimQueueEntry<T> {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (request_key, bytes) = T::from_bytes(bytes)?;
        let (amount, bytes) = U512::from_bytes(bytes)?;
        let entry = ClaimQueueEntry {
            request_key,
            amount,
        };
        Ok((entry, bytes))
    }
}

impl<T: RequestKey> ToBytes for ClaimQueueEntry<T> {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.request_key.to_bytes()?.into_iter())
            .chain(self.amount.to_bytes()?)
            .collect())
    }
}

impl<T: RequestKey> CLTyped for ClaimQueueEntry<T> {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
