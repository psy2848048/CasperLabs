use alloc::vec::Vec;
use types::{account::PublicKey, bytesrepr::ToBytes, Key};

pub const TOTAL_MINT_SUPPLY: u8 = 0;
pub const UNBOND_REQUEST_QUEUE: u8 = 1;
pub const UNDELEGATE_REQUEST_QUEUE: u8 = 2;
pub const REDELEGATE_REQUEST_QUEUE: u8 = 3;
pub const CLAIM_REQUESTS: u8 = 4;

// a single vote: (ACTION_PREFIX_VOTING + voter_pubkey + dapp_addr, amount)
const ACTION_PREFIX_STAKE: u8 = 1;
const ACTION_PREFIX_VOTING: u8 = 2;
const ACTION_PREFIX_VOTED: u8 = 3;
// 1 - 31 are reserved for ACTION_PREFIX
const PREFIX_COMMISSION: u8 = 32;
const PREFIX_REWARD: u8 = 33;

pub fn bonding_amount_key(user: &PublicKey) -> Vec<u8> {
    let mut ret = Vec::with_capacity(1 + user.as_bytes().len());
    ret.push(ACTION_PREFIX_STAKE);
    ret.extend(user.as_bytes());
    ret
}

pub fn voting_amount_key(user: &PublicKey) -> Vec<u8> {
    let mut ret = Vec::with_capacity(1 + user.as_bytes().len());
    ret.push(ACTION_PREFIX_VOTING);
    ret.extend(user.as_bytes());
    ret
}

pub fn voted_amount_key(dapp: &Key) -> Vec<u8> {
    let mut ret = Vec::with_capacity(1 + dapp.serialized_length());
    ret.push(ACTION_PREFIX_VOTED);
    ret.extend(
        dapp.to_bytes()
            .expect("Key serialization cannot fail")
            .into_iter(),
    );
    ret
}

pub fn vote_key(voter: &PublicKey, dapp: &Key) -> Vec<u8> {
    let mut ret = Vec::with_capacity(1 + voter.as_bytes().len() + dapp.serialized_length());
    ret.push(ACTION_PREFIX_VOTING);
    ret.extend(voter.as_bytes());
    ret.extend(
        dapp.to_bytes()
            .expect("Key serialization cannot fail")
            .into_iter(),
    );
    ret
}

pub fn commission_amount_key(validator: &PublicKey) -> Vec<u8> {
    let mut ret = Vec::with_capacity(1 + validator.as_bytes().len());
    ret.push(PREFIX_COMMISSION);
    ret.extend(validator.as_bytes());
    ret
}

pub fn reward_amount_key(user: &PublicKey) -> Vec<u8> {
    let mut ret = Vec::with_capacity(1 + user.as_bytes().len());
    ret.push(PREFIX_REWARD);
    ret.extend(user.as_bytes());
    ret
}
