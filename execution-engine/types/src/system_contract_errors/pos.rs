//! Home of the Proof of Stake contract's [`Error`] type.

use core::result;

/// Errors which can occur while executing the Proof of Stake contract.
#[derive(Debug, PartialEq)]
// TODO: Split this up into user errors vs. system errors.
#[repr(u8)]
pub enum Error {
    // ===== User errors =====
    /// The given validator is not bonded.
    NotBonded = 0,
    /// There are too many bonding or unbonding attempts already enqueued to allow more.
    TooManyEventsInQueue,
    /// At least one validator must remain bonded.
    CannotUnbondLastValidator,
    /// Failed to bond or unbond as this would have resulted in exceeding the maximum allowed
    /// difference between the largest and smallest stakes.
    SpreadTooHigh,
    /// The given validator already has a bond or unbond attempt enqueued.
    MultipleRequests,
    /// Attempted to bond with a stake which was too small.
    BondTooSmall,
    /// Attempted to bond with a stake which was too large.
    BondTooLarge,
    /// Attempted to unbond an amount which was too large.
    UnbondTooLarge,
    /// While bonding, the transfer from source purse to the Proof of Stake internal purse failed.
    BondTransferFailed,
    /// While unbonding, the transfer from the Proof of Stake internal purse to the destination
    /// purse failed.
    UnbondTransferFailed,
    // ===== System errors =====
    /// Internal error: a [`BlockTime`](crate::BlockTime) was unexpectedly out of sequence.
    TimeWentBackwards,
    /// Internal error: stakes were unexpectedly empty.
    StakesNotFound,
    /// Internal error: the PoS contract's payment purse wasn't found.
    PaymentPurseNotFound,
    /// Internal error: the PoS contract's payment purse key was the wrong type.
    PaymentPurseKeyUnexpectedType,
    /// Internal error: couldn't retrieve the balance for the PoS contract's payment purse.
    PaymentPurseBalanceNotFound,
    /// Internal error: the PoS contract's bonding purse wasn't found.
    BondingPurseNotFound,
    /// Internal error: the PoS contract's bonding purse key was the wrong type.
    BondingPurseKeyUnexpectedType,
    /// Internal error: the PoS contract's refund purse key was the wrong type.
    RefundPurseKeyUnexpectedType,
    /// Internal error: the PoS contract's rewards purse wasn't found.
    RewardsPurseNotFound,
    /// Internal error: the PoS contract's rewards purse key was the wrong type.
    RewardsPurseKeyUnexpectedType,
    // TODO: Put these in their own enum, and wrap them separately in `BondingError` and
    //       `UnbondingError`.
    /// Internal error: failed to deserialize the stake's key.
    StakesKeyDeserializationFailed,
    /// Internal error: failed to deserialize the stake's balance.
    StakesDeserializationFailed,
    /// The invoked PoS function can only be called by system contracts, but was called by a user
    /// contract.
    SystemFunctionCalledByUserAccount,
    /// Internal error: while finalizing payment, the amount spent exceeded the amount available.
    InsufficientPaymentForAmountSpent,
    /// Internal error: while finalizing payment, failed to pay the validators (the transfer from
    /// the PoS contract's payment purse to rewards purse failed).
    FailedTransferToRewardsPurse,
    /// Internal error: while finalizing payment, failed to refund the caller's purse (the transfer
    /// from the PoS contract's payment purse to refund purse or account's main purse failed).
    FailedTransferToAccountPurse,
    /// PoS contract's "set_refund_purse" method can only be called by the payment code of a
    /// deploy, but was called by the session code.
    SetRefundPurseCalledOutsidePayment,

    // ===== HDAC PoS errors =====
    /// The given delegation relation(delgator-validator) does not exist.
    NotDelegated, // = 27
    /// Attempted to undelegate an amount which was too large.
    UndelegateTooLarge, // = 28
    /// Attempted to self-redelegate.
    SelfRedelegation, // = 29

    /// Internal error: delegations are unexpectedly empty.
    DelegationsNotFound, // = 30
    /// Internal error: attempted to use unsupported function.
    NotSupportedFunc, // = 31
    /// Internal error: failed to deserialize the delegation's key
    DelegationsKeyDeserializationFailed, // = 32
    /// Internal error: failed to deserialize the delegation's amount
    DelegationsDeserializationFailed, // = 33

    /// Internal error: delegations are unexpectedly empty.
    VotesNotFound, // = 34
    /// Internal error: failed to deserialize the delegation's key
    VoteKeyDeserializationFailed, // = 35
    /// Internal error: failed to deserialize the delegation's amount
    VotesDeserializationFailed, // = 36
    /// Internal error: No vote record
    NotVoted, // = 37
    /// Attempted to unvote with too big number to occur overflow
    UnvoteTooLarge, // = 38
    /// Attempted to vote too large number
    VoteTooLarge, // = 39

    /// Internal error: failed to deserialize the validator's key
    CommissionKeyDeserializationFailed, // = 40
    /// Internal error: failed to deserialize the validator's balance
    CommissionBalanceDeserializationFailed, // = 41
    /// Internal error: failed to issue a new purse for commission
    CommissionPurseNotFound, // = 42
    /// Internal error: no commission record
    CommissionNotFound, // = 43
    /// No claim record of commission
    CommissionClaimRecordNotFound, // = 44
    /// Internal error: Too large claim than you earned
    CommissionClaimTooLarge, // = 45
    /// Internal error: failed to deserialize the user's key
    RewardKeyDeserializationFailed, // = 46
    /// Internal error: failed to deserialize the user's balance
    RewardBalanceDeserializationFailed, // = 47
    /// Internal error: failed to issue a new purse for commission
    RewardPurseNotFound, // = 48
    /// Internal error: no reward record in table
    RewardNotFound, // = 49
    /// Internal error: claim reward than expected
    RewardClaimTooLarge, // = 50
    /// Try to claim although user has no reward
    RewardClaimRecordNotFound, // = 51
    /// Internal error: No total supply
    NoTotalSupply, // = 52
    /// Error while parsing between u512 and stringed integer
    UintParsingError, // = 53
    /// Internal error: Deserialization failed about total supply
    TotalSupplyDeserializationFailed, // = 54
    /// Internal error: No commission object
    NoCommission, // = 55
    /// Internal error: No reward object
    NoReward, // = 56
}

/// An alias for `Result<T, pos::Error>`.
pub type Result<T> = result::Result<T, Error>;

// This error type is not intended to be used by third party crates.
#[doc(hidden)]
pub enum PurseLookupError {
    KeyNotFound,
    KeyUnexpectedType,
}

// This error type is not intended to be used by third party crates.
#[doc(hidden)]
impl PurseLookupError {
    pub fn bonding(err: PurseLookupError) -> Error {
        match err {
            PurseLookupError::KeyNotFound => Error::BondingPurseNotFound,
            PurseLookupError::KeyUnexpectedType => Error::BondingPurseKeyUnexpectedType,
        }
    }

    pub fn payment(err: PurseLookupError) -> Error {
        match err {
            PurseLookupError::KeyNotFound => Error::PaymentPurseNotFound,
            PurseLookupError::KeyUnexpectedType => Error::PaymentPurseKeyUnexpectedType,
        }
    }

    pub fn rewards(err: PurseLookupError) -> Error {
        match err {
            PurseLookupError::KeyNotFound => Error::RewardsPurseNotFound,
            PurseLookupError::KeyUnexpectedType => Error::RewardsPurseKeyUnexpectedType,
        }
    }
}
