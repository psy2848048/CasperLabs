pub(crate) mod uref_names {
    pub const POS_BONDING_PURSE: &str = "pos_bonding_purse";
    pub const POS_REWARD_PURSE: &str = "pos_rewards_purse";
    pub const POS_PAYMENT_PURSE: &str = "pos_payment_purse";
    pub const POS_PREMINT_PURSE: &str = "pos_premint_purse";
    pub const _POS_COMMUNITY_PURSE: &str = "pos_community_purse";
}

pub(crate) mod methods {
    pub const METHOD_INSTALL_GENESIS_STATES: &str = "install_genesis_states";
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
    pub const MAX_VALIDATORS: usize = 100;

    #[cfg(test)]
    pub const UNBONDING_DELAY_IN_SEC: u64 = 0;
    #[cfg(any(not(test), feature = "pop-delay-test"))]
    pub const UNBONDING_DELAY_IN_SEC: u64 = 2 * 24 * 60 * 60;
    pub const UNDELEGATING_DELAY_IN_SEC: u64 = 0;

    pub const BLOCK_PRODUCING_SEC: i64 = 5_i64;
    pub const MAX_SUPPLY: u64 = 2_800_000_000_u64;
    pub const BIGSUN_TO_HDAC: u64 = 1_000_000_000_000_000_000_u64;
    pub const VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE: i64 = 30_i64;
    pub const INFLATION_RATE: i64 = 488_i64;
}
