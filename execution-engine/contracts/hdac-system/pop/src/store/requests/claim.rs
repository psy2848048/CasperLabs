use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

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
