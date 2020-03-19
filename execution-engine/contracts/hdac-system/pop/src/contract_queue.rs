use alloc::vec::Vec;
use core::result;

use contract::contract_api::storage;
use proof_of_stake::{self, QueueProvider};
use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use crate::{
    constants::local_keys::{REDELEGATE_REQUEST_QUEUE, UNDELEGATE_REQUEST_QUEUE},
    request_queue::{Request, RequestQueue},
};

pub struct ContractQueue;

impl ContractQueue {
    pub fn read_undelegate_requests() -> UndelegateRequestQueue {
        storage::read_local(&UNDELEGATE_REQUEST_QUEUE)
            .unwrap_or_default()
            .unwrap_or_default()
    }

    pub fn read_redelegate_requests() -> RedelegateRequestQueue {
        storage::read_local(&REDELEGATE_REQUEST_QUEUE)
            .unwrap_or_default()
            .unwrap_or_default()
    }

    pub fn write_undelegate_requests(queue: UndelegateRequestQueue) {
        storage::write_local(UNDELEGATE_REQUEST_QUEUE, queue);
    }

    pub fn write_redelegate_requests(queue: RedelegateRequestQueue) {
        storage::write_local(REDELEGATE_REQUEST_QUEUE, queue);
    }
}

type UndelegateRequestQueue = RequestQueue<UndelegateRequest>;
type RedelegateRequestQueue = RequestQueue<RedelegateRequest>;

#[derive(Clone, Copy)]
pub struct UndelegateRequest {
    pub delegator: PublicKey,
    pub validator: PublicKey,
    pub amount: U512,
}

impl Default for RequestQueue<UndelegateRequest> {
    fn default() -> Self {
        RequestQueue::<UndelegateRequest> { 0: Vec::new() }
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

#[derive(Clone, Copy)]
pub struct RedelegateRequest {
    pub delegator: PublicKey,
    pub src_validator: PublicKey,
    pub dest_validator: PublicKey,
    pub amount: U512,
}

impl Default for RequestQueue<RedelegateRequest> {
    fn default() -> Self {
        RequestQueue::<RedelegateRequest> { 0: Vec::new() }
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

// TODO: remove QueueProvider
// Currently, we are utilizing the default implemention of the Proof-of-Stake crate,
// so we need to add a dummy implemention to meet trait contraint.
impl QueueProvider for ContractQueue {
    fn read_bonding() -> proof_of_stake::Queue {
        unimplemented!()
    }

    fn read_unbonding() -> proof_of_stake::Queue {
        unimplemented!()
    }

    fn write_bonding(_: proof_of_stake::Queue) {
        unimplemented!()
    }

    fn write_unbonding(_: proof_of_stake::Queue) {
        unimplemented!()
    }
}
