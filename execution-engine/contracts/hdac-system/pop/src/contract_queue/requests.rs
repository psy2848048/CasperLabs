use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use super::request_queue::Request;

#[derive(Clone, Copy)]
pub struct UndelegateRequest {
    pub delegator: PublicKey,
    pub validator: PublicKey,
    pub amount: U512,
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

#[derive(Clone, Copy)]
pub struct RedelegateRequest {
    pub delegator: PublicKey,
    pub src_validator: PublicKey,
    pub dest_validator: PublicKey,
    pub amount: U512,
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
