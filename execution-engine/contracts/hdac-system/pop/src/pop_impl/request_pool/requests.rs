use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use super::delegation_queue::RequestKey;

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
            delegator: PublicKey::ed25519_from([0u8; 32]),
            validator: PublicKey::ed25519_from([0u8; 32]),
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
    fn serialized_length(&self) -> usize {
        self.delegator.serialized_length() + self.validator.serialized_length()
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
            delegator: PublicKey::ed25519_from([0u8; 32]),
            src_validator: PublicKey::ed25519_from([0u8; 32]),
            dest_validator: PublicKey::ed25519_from([0u8; 32]),
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
    fn serialized_length(&self) -> usize {
        self.delegator.serialized_length()
            + self.src_validator.serialized_length()
            + self.dest_validator.serialized_length()
    }
}

impl CLTyped for RedelegateRequestKey {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

#[derive(Clone, Debug, Copy)]
pub enum ClaimRequest {
    Commission(PublicKey, U512, U512),
    Reward(PublicKey, U512, U512),
}

const COMMISSION_ID: u8 = 1;
const REWARD_ID: u8 = 2;

impl Default for ClaimRequest {
    fn default() -> Self {
        ClaimRequest::Commission(
            PublicKey::ed25519_from([0u8; 32]),
            U512::default(),
            U512::default(),
        )
    }
}

impl FromBytes for ClaimRequest {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (claim_type, rest): (u8, &[u8]) = FromBytes::from_bytes(bytes)?;
        let (pubkey, rest): (PublicKey, &[u8]) = FromBytes::from_bytes(rest)?;
        let (inflation_amount, rest): (U512, &[u8]) = FromBytes::from_bytes(rest)?;
        let (fare_amount, rest): (U512, &[u8]) = FromBytes::from_bytes(rest)?;
        match claim_type {
            COMMISSION_ID => Ok((
                ClaimRequest::Commission(pubkey, inflation_amount, fare_amount),
                rest,
            )),
            REWARD_ID => Ok((
                ClaimRequest::Reward(pubkey, inflation_amount, fare_amount),
                rest,
            )),
            _ => Err(bytesrepr::Error::Formatting),
        }
    }
}

impl ToBytes for ClaimRequest {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let mut res = Vec::new();
        match self {
            ClaimRequest::Commission(pubkey, inflation_amount, fare_amount) => {
                res.push(COMMISSION_ID);
                res.extend(pubkey.to_bytes()?);
                res.extend(inflation_amount.to_bytes()?);
                res.extend(fare_amount.to_bytes()?);
            }
            ClaimRequest::Reward(pubkey, inflation_amount, fare_amount) => {
                res.push(REWARD_ID);
                res.extend(pubkey.to_bytes()?);
                res.extend(inflation_amount.to_bytes()?);
                res.extend(fare_amount.to_bytes()?);
            }
        }
        Ok(res)
    }
    fn serialized_length(&self) -> usize {
        match self {
            ClaimRequest::Commission(pubkey, inflation_amount, fare_amount)
            | ClaimRequest::Reward(pubkey, inflation_amount, fare_amount) => {
                pubkey.serialized_length()
                    + inflation_amount.serialized_length()
                    + fare_amount.serialized_length()
            }
        }
    }
}

impl CLTyped for ClaimRequest {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
