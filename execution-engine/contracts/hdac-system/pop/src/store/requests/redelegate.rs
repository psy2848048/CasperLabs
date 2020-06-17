use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use crate::duration_queue::DurationQueueItem;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RedelegateRequest {
    pub delegator: PublicKey,
    pub src_validator: PublicKey,
    pub dest_validator: PublicKey,
    pub maybe_amount: Option<U512>,
}

impl DurationQueueItem for RedelegateRequest {}

impl Default for RedelegateRequest {
    fn default() -> Self {
        RedelegateRequest {
            delegator: PublicKey::ed25519_from([0u8; 32]),
            src_validator: PublicKey::ed25519_from([0u8; 32]),
            dest_validator: PublicKey::ed25519_from([0u8; 32]),
            maybe_amount: None,
        }
    }
}

impl FromBytes for RedelegateRequest {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (src_validator, bytes) = PublicKey::from_bytes(bytes)?;
        let (dest_validator, bytes) = PublicKey::from_bytes(bytes)?;
        let (maybe_amount, bytes): (Option<U512>, &[u8]) = FromBytes::from_bytes(bytes)?;
        Ok((
            RedelegateRequest {
                delegator,
                src_validator,
                dest_validator,
                maybe_amount,
            },
            bytes,
        ))
    }
}

impl ToBytes for RedelegateRequest {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.src_validator.to_bytes()?)
            .chain(self.dest_validator.to_bytes()?)
            .chain(self.maybe_amount.to_bytes()?)
            .collect())
    }
    fn serialized_length(&self) -> usize {
        self.delegator.serialized_length()
            + self.src_validator.serialized_length()
            + self.dest_validator.serialized_length()
            + self.maybe_amount.serialized_length()
    }
}

impl CLTyped for RedelegateRequest {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
