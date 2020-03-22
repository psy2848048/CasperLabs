use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use super::request_queue::Request;

pub type DelegateRequest = UndelegateRequest;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UndelegateRequest {
    delegator: PublicKey,
    validator: PublicKey,
    amount: U512,
}

impl UndelegateRequest {
    pub fn new(delegator: PublicKey, validator: PublicKey, amount: U512) -> Self {
        UndelegateRequest {
            delegator,
            validator,
            amount,
        }
    }
}

impl Default for UndelegateRequest {
    fn default() -> Self {
        UndelegateRequest {
            delegator: PublicKey::from([0u8; 32]),
            validator: PublicKey::from([0u8; 32]),
            amount: U512::from(0),
        }
    }
}

impl Request for UndelegateRequest {
    fn is_same(&self, rhs: &Self) -> bool {
        self.delegator == rhs.delegator && self.validator == rhs.validator
    }
}

impl FromBytes for UndelegateRequest {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (validator, bytes) = PublicKey::from_bytes(bytes)?;
        let (amount, bytes) = U512::from_bytes(bytes)?;
        let entry = UndelegateRequest {
            delegator,
            validator,
            amount,
        };
        Ok((entry, bytes))
    }
}

impl ToBytes for UndelegateRequest {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.validator.to_bytes()?)
            .chain(self.amount.to_bytes()?)
            .collect())
    }
}

impl CLTyped for UndelegateRequest {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RedelegateRequest {
    delegator: PublicKey,
    src_validator: PublicKey,
    dest_validator: PublicKey,
    amount: U512,
}

impl RedelegateRequest {
    pub fn new(
        delegator: PublicKey,
        src_validator: PublicKey,
        dest_validator: PublicKey,
        amount: U512,
    ) -> Self {
        RedelegateRequest {
            delegator,
            src_validator,
            dest_validator,
            amount,
        }
    }
}

impl Default for RedelegateRequest {
    fn default() -> Self {
        RedelegateRequest {
            delegator: PublicKey::from([0u8; 32]),
            src_validator: PublicKey::from([0u8; 32]),
            dest_validator: PublicKey::from([0u8; 32]),
            amount: U512::from(0),
        }
    }
}

impl Request for RedelegateRequest {
    fn is_same(&self, rhs: &Self) -> bool {
        self.delegator == rhs.delegator
            && self.src_validator == rhs.src_validator
            && self.dest_validator == rhs.dest_validator
    }
}

impl FromBytes for RedelegateRequest {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (src_validator, bytes) = PublicKey::from_bytes(bytes)?;
        let (dest_validator, bytes) = PublicKey::from_bytes(bytes)?;
        let (amount, bytes) = U512::from_bytes(bytes)?;
        let entry = RedelegateRequest {
            delegator,
            src_validator,
            dest_validator,
            amount,
        };
        Ok((entry, bytes))
    }
}

impl ToBytes for RedelegateRequest {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.src_validator.to_bytes()?)
            .chain(self.dest_validator.to_bytes()?)
            .chain(self.amount.to_bytes()?)
            .collect())
    }
}

impl CLTyped for RedelegateRequest {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
