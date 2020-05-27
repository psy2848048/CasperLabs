use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use super::duration_queue::DurationQueueItem;

#[derive(Clone, Copy)]
pub struct UnbondRequest {
    requester: PublicKey,
    maybe_amount: Option<U512>,
}

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

#[derive(Clone, Debug, Copy)]
pub enum ClaimRequest {
    Commission(PublicKey, U512),
    Reward(PublicKey, U512),
}

const COMMISSION_ID: u8 = 1;
const REWARD_ID: u8 = 2;

impl Default for ClaimRequest {
    fn default() -> Self {
        ClaimRequest::Commission(PublicKey::ed25519_from([0u8; 32]), U512::default())
    }
}

impl FromBytes for ClaimRequest {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (claim_type, rest): (u8, &[u8]) = FromBytes::from_bytes(bytes)?;
        let (pubkey, rest): (PublicKey, &[u8]) = FromBytes::from_bytes(rest)?;
        let (amount, rest): (U512, &[u8]) = FromBytes::from_bytes(rest)?;
        match claim_type {
            COMMISSION_ID => Ok((ClaimRequest::Commission(pubkey, amount), rest)),
            REWARD_ID => Ok((ClaimRequest::Reward(pubkey, amount), rest)),
            _ => Err(bytesrepr::Error::Formatting),
        }
    }
}

impl ToBytes for ClaimRequest {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let mut res = Vec::new();
        match self {
            ClaimRequest::Commission(pubkey, amount) => {
                res.push(COMMISSION_ID);
                res.extend(pubkey.to_bytes()?);
                res.extend(amount.to_bytes()?);
            }
            ClaimRequest::Reward(pubkey, amount) => {
                res.push(REWARD_ID);
                res.extend(pubkey.to_bytes()?);
                res.extend(amount.to_bytes()?);
            }
        }
        Ok(res)
    }
    fn serialized_length(&self) -> usize {
        match self {
            ClaimRequest::Commission(pubkey, amount) | ClaimRequest::Reward(pubkey, amount) => {
                pubkey.serialized_length() + amount.serialized_length()
            }
        }
    }
}

impl CLTyped for ClaimRequest {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
