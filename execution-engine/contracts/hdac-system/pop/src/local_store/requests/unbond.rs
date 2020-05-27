use alloc::vec::Vec;
use core::result;

use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, U512,
};

use crate::duration_queue::DurationQueueItem;

#[derive(Clone, Copy)]
pub struct UnbondRequest {
    requester: PublicKey,
    maybe_amount: Option<U512>,
}

// impl DurationQueueItem for UnbondRequest {}
