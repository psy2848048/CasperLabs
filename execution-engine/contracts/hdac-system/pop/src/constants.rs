pub(crate) mod local_keys {
    pub const BONDING_KEY: u8 = 1;
    pub const UNBONDING_KEY: u8 = 2;
}

pub(crate) mod uref_names {
    pub const POS_BONDING_PURSE: &str = "pos_bonding_purse";
}

pub(crate) mod methods {
    pub const METHOD_BOND: &str = "bond";
    pub const METHOD_UNBOND: &str = "unbond";
    pub const METHOD_STEP: &str = "step";
    pub const METHOD_GET_PAYMENT_PURSE: &str = "get_payment_purse";
    pub const METHOD_SET_REFUND_PURSE: &str = "set_refund_purse";
    pub const METHOD_GET_REFUND_PURSE: &str = "get_refund_purse";
    pub const METHOD_FINALIZE_PAYMENT: &str = "finalize_payment";

    pub const METHOD_DELEGATE: &str = "delegate";
    pub const METHOD_UNDELEGATE: &str = "undelegate";
    pub const METHOD_REDELEGATE: &str = "redelegate";
}
