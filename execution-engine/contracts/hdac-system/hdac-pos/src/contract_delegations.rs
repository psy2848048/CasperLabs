use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped,
};

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct DelegationKey {
    pub delegator: PublicKey,
    pub validator: PublicKey,
}

impl FromBytes for DelegationKey {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (validator, bytes) = PublicKey::from_bytes(bytes)?;
        let entry = DelegationKey {
            delegator,
            validator,
        };
        Ok((entry, bytes))
    }
}

impl ToBytes for DelegationKey {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.validator.to_bytes()?)
            .collect())
    }
}

impl CLTyped for DelegationKey {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
