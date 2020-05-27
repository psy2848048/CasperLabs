use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use crate::duration_queue::DurationQueueItem;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UndelegateRequest {
    pub delegator: PublicKey,
    pub validator: PublicKey,
    pub maybe_amount: Option<U512>,
}

impl DurationQueueItem for UndelegateRequest {}

impl Default for UndelegateRequest {
    fn default() -> Self {
        UndelegateRequest {
            delegator: PublicKey::ed25519_from([0u8; 32]),
            validator: PublicKey::ed25519_from([0u8; 32]),
            maybe_amount: None,
        }
    }
}

impl FromBytes for UndelegateRequest {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (validator, bytes) = PublicKey::from_bytes(bytes)?;
        let (maybe_amount, bytes): (Option<U512>, &[u8]) = FromBytes::from_bytes(bytes)?;

        Ok((
            UndelegateRequest {
                delegator,
                validator,
                maybe_amount,
            },
            bytes,
        ))
    }
}

impl ToBytes for UndelegateRequest {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.validator.to_bytes()?)
            .chain(self.maybe_amount.to_bytes()?)
            .collect())
    }
    fn serialized_length(&self) -> usize {
        self.delegator.serialized_length()
            + self.validator.serialized_length()
            + self.maybe_amount.serialized_length()
    }
}

impl CLTyped for UndelegateRequest {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
