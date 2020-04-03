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

#[repr(u8)]
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ClaimKeyType {
    Error = 0,
    Commission = 1,
    Reward = 2,
}

impl From<u8> for ClaimKeyType {
    fn from(key_byte: u8) -> Self {
        match key_byte {
            1u8 => ClaimKeyType::Commission,
            2u8 => ClaimKeyType::Reward,
            _ => ClaimKeyType::Error,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct ClaimRequestKey {
    pub key_type: ClaimKeyType, // := commission, reward
    pub pubkey: PublicKey,
}

impl ClaimRequestKey {
    pub fn new(key_type: ClaimKeyType, pubkey: PublicKey) -> Self {
        ClaimRequestKey {
            key_type,
            pubkey,
        }
    }
}

impl Default for ClaimRequestKey {
    fn default() -> Self {
        ClaimRequestKey {
            key_type: ClaimKeyType::Commission,
            pubkey: PublicKey::from([0u8; 32]),
        }
    }
}

impl RequestKey for ClaimRequestKey {}

impl FromBytes for ClaimRequestKey {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let key_type: ClaimKeyType = bytes[0].into();
        let bytes = &bytes[1..];
        let (pubkey, bytes) = PublicKey::from_bytes(bytes)?;
        let entry = ClaimRequestKey {
            key_type,
            pubkey,
        };
        Ok((entry, bytes))
    }
}

impl ToBytes for ClaimRequestKey {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let mut res: Vec<u8> = Vec::new();
        res.push(self.key_type as u8);
        Ok((res.into_iter())
            .chain(self.pubkey.to_bytes()?)
            .collect())
    }
}

impl CLTyped for ClaimRequestKey {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
