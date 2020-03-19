use alloc::vec::Vec;

use types::{
    system_contract_errors::pos::{Error, Result},
    BlockTime,
};

#[derive(Clone, Default)]
pub struct RequestQueue<T: Request + Clone + Copy>(pub Vec<RequestQueueEntry<T>>);

#[derive(Clone, Copy, Debug)]
pub struct RequestQueueEntry<T: Request + Clone + Copy> {
    pub request: T,
    pub timestamp: BlockTime,
}

pub trait Request {
    fn is_same(&self, rhs: &Self) -> bool;
}

impl<T: Request + Clone + Copy> RequestQueue<T> {
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
