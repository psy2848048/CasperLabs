use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use crate::duration_queue::DurationQueueItem;

#[derive(Clone, Copy)]
pub struct UnbondRequest {
    pub requester: PublicKey,
    pub maybe_amount: Option<U512>,
}

impl DurationQueueItem for UnbondRequest {}

impl Default for UnbondRequest {
    fn default() -> Self {
        UnbondRequest {
            requester: PublicKey::ed25519_from([0u8; 32]),
            maybe_amount: None,
        }
    }
}

impl FromBytes for UnbondRequest {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (requester, bytes) = PublicKey::from_bytes(bytes)?;
        let (maybe_amount, bytes): (Option<U512>, &[u8]) = FromBytes::from_bytes(bytes)?;
        Ok((
            UnbondRequest {
                requester,
                maybe_amount,
            },
            bytes,
        ))
    }
}

impl ToBytes for UnbondRequest {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.requester.to_bytes()?.into_iter())
            .chain(self.maybe_amount.to_bytes()?)
            .collect())
    }
    fn serialized_length(&self) -> usize {
        self.requester.serialized_length() + self.maybe_amount.serialized_length()
    }
}

impl CLTyped for UnbondRequest {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
