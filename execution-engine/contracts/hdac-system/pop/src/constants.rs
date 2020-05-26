pub(crate) mod local_keys {
    use alloc::vec::Vec;
    use types::{account::PublicKey, bytesrepr::ToBytes, Key};
    pub const UNBOND_REQUEST_QUEUE: u8 = 1;
    pub const UNDELEGATE_REQUEST_QUEUE: u8 = 2;
    pub const REDELEGATE_REQUEST_QUEUE: u8 = 3;
    pub const CLAIM_REQUESTS: u8 = 4;

    // a single delegation: (ACTION_PREFIX_DELEGATING + delegator_pubkey + validator_pubkey, amount)
    // a single vote: (ACTION_PREFIX_VOTING + voter_pubkey + dapp_addr, amount)
    const ACTION_PREFIX_STAKE: u8 = 1;
    const ACTION_PREFIX_DELEGATING: u8 = 2;
    const ACTION_PREFIX_DELEGATED: u8 = 3;
    const ACTION_PREFIX_VOTING: u8 = 4;
    const ACTION_PREFIX_VOTED: u8 = 5;

    pub fn staking_amount_key(user: PublicKey) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + user.as_bytes().len());
        ret.push(ACTION_PREFIX_STAKE);
        ret.extend(user.as_bytes());
        ret
    }

    pub fn delegating_amount_key(user: PublicKey) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + user.as_bytes().len());
        ret.push(ACTION_PREFIX_DELEGATING);
        ret.extend(user.as_bytes());
        ret
    }

    pub fn delegated_amount_key(user: PublicKey) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + user.as_bytes().len());
        ret.push(ACTION_PREFIX_DELEGATED);
        ret.extend(user.as_bytes());
        ret
    }

    pub fn voting_amount_key(user: PublicKey) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + user.as_bytes().len());
        ret.push(ACTION_PREFIX_VOTING);
        ret.extend(user.as_bytes());
        ret
    }

    pub fn voted_amount_key(dapp: Key) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + dapp.serialized_length());
        ret.push(ACTION_PREFIX_VOTED);
        ret.extend(
            dapp.to_bytes()
                .expect("Key serialization cannot fail")
                .into_iter(),
        );
        ret
    }

    pub fn delegation_key(delegator: PublicKey, validator: PublicKey) -> Vec<u8> {
        let mut ret = Vec::with_capacity(1 + 2 * delegator.as_bytes().len());
        ret.push(ACTION_PREFIX_DELEGATING);
        ret.extend(delegator.as_bytes());
        ret.extend(validator.as_bytes());
        ret
    }

    pub fn vote_key(voter: PublicKey, dapp: Key) -> Vec<u8> {
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
}

pub(crate) mod uref_names {
    pub const POS_BONDING_PURSE: &str = "pos_bonding_purse";
    pub const POS_REWARD_PURSE: &str = "pos_rewards_purse";
    pub const POS_PAYMENT_PURSE: &str = "pos_payment_purse";
    pub const POS_COMMISSION_PURSE: &str = "pos_commission_purse";
    pub const POS_COMMUNITY_PURSE: &str = "pos_community_purse";
}

pub(crate) mod methods {
    pub const METHOD_BOND: &str = "bond";
    pub const METHOD_UNBOND: &str = "unbond";
    pub const METHOD_STEP: &str = "step";
    pub const METHOD_GET_PAYMENT_PURSE: &str = "get_payment_purse";
    pub const METHOD_FINALIZE_PAYMENT: &str = "finalize_payment";

    pub const METHOD_DELEGATE: &str = "delegate";
    pub const METHOD_UNDELEGATE: &str = "undelegate";
    pub const METHOD_REDELEGATE: &str = "redelegate";
    pub const METHOD_VOTE: &str = "vote";
    pub const METHOD_UNVOTE: &str = "unvote";
    pub const METHOD_CLAIM_COMMISSION: &str = "claim_commission";
    pub const METHOD_CLAIM_REWARD: &str = "claim_reward";
}

pub(crate) mod sys_params {
    pub const SYSTEM_ACCOUNT: [u8; 32] = [0u8; 32];

    pub const UNBONDING_DELAY: u64 = 0;
    pub const UNDELEGATING_DELAY: u64 = 0;

    pub const BLOCK_PRODUCING_PER_SEC: i64 = 2_i64;
    pub const MAX_SUPPLY: u64 = 999_999_999_999_u64; // TODO: Should change the value before mainnet launce
    pub const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;
    pub const VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE: i64 = 30_i64;
}
