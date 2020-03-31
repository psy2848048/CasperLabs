use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped,
};

use super::request_queue::RequestKey;

pub type DelegateRequestKey = UndelegateRequestKey;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UndelegateRequestKey {
    pub delegator: PublicKey,
    pub validator: PublicKey,
}

impl UndelegateRequestKey {
    pub fn new(delegator: PublicKey, validator: PublicKey) -> Self {
        UndelegateRequestKey {
            delegator,
            validator,
        }
    }
}

impl Default for UndelegateRequestKey {
    fn default() -> Self {
        UndelegateRequestKey {
            delegator: PublicKey::from([0u8; 32]),
            validator: PublicKey::from([0u8; 32]),
        }
    }
}

impl RequestKey for UndelegateRequestKey {}

impl FromBytes for UndelegateRequestKey {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (validator, bytes) = PublicKey::from_bytes(bytes)?;
        let entry = UndelegateRequestKey {
            delegator,
            validator,
        };
        Ok((entry, bytes))
    }
}

impl ToBytes for UndelegateRequestKey {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.validator.to_bytes()?)
            .collect())
    }
}

impl CLTyped for UndelegateRequestKey {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RedelegateRequestKey {
    pub delegator: PublicKey,
    pub src_validator: PublicKey,
    pub dest_validator: PublicKey,
}

impl RedelegateRequestKey {
    pub fn new(delegator: PublicKey, src_validator: PublicKey, dest_validator: PublicKey) -> Self {
        RedelegateRequestKey {
            delegator,
            src_validator,
            dest_validator,
        }
    }
}

impl Default for RedelegateRequestKey {
    fn default() -> Self {
        RedelegateRequestKey {
            delegator: PublicKey::from([0u8; 32]),
            src_validator: PublicKey::from([0u8; 32]),
            dest_validator: PublicKey::from([0u8; 32]),
        }
    }
}

impl RequestKey for RedelegateRequestKey {}

impl FromBytes for RedelegateRequestKey {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (src_validator, bytes) = PublicKey::from_bytes(bytes)?;
        let (dest_validator, bytes) = PublicKey::from_bytes(bytes)?;
        let entry = RedelegateRequestKey {
            delegator,
            src_validator,
            dest_validator,
        };
        Ok((entry, bytes))
    }
}

impl ToBytes for RedelegateRequestKey {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.src_validator.to_bytes()?)
            .chain(self.dest_validator.to_bytes()?)
            .collect())
    }
}

impl CLTyped for RedelegateRequestKey {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
