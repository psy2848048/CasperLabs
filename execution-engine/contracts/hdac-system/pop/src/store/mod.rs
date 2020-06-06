mod local_keys;
mod requests;

use alloc::vec::Vec;

use contract::contract_api::storage;
use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

pub use requests::{ClaimRequest, RedelegateRequest, UnbondRequest, UndelegateRequest};

use crate::duration_queue::DurationQueue;

pub fn read_unbond_requests() -> DurationQueue<UnbondRequest> {
    storage::read_local(&local_keys::UNBOND_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_unbond_requests(queue: DurationQueue<UnbondRequest>) {
    storage::write_local(local_keys::UNBOND_REQUEST_QUEUE, queue);
}

pub fn read_undelegation_requests() -> DurationQueue<UndelegateRequest> {
    storage::read_local(&local_keys::UNDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_undelegation_requests(queue: DurationQueue<UndelegateRequest>) {
    storage::write_local(local_keys::UNDELEGATE_REQUEST_QUEUE, queue);
}

pub fn read_redelegation_requests() -> DurationQueue<RedelegateRequest> {
    storage::read_local(&local_keys::REDELEGATE_REQUEST_QUEUE)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_redelegation_requests(queue: DurationQueue<RedelegateRequest>) {
    storage::write_local(local_keys::REDELEGATE_REQUEST_QUEUE, queue);
}

pub fn read_claim_requests() -> Vec<ClaimRequest> {
    storage::read_local(&local_keys::CLAIM_REQUESTS)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn write_claim_requests(list: Vec<ClaimRequest>) {
    storage::write_local(local_keys::CLAIM_REQUESTS, list);
}

pub fn read_bonding_amount(user: PublicKey) -> U512 {
    let key = local_keys::bonding_amount_key(user);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn bond(user: PublicKey, amount: U512) {
    let key = local_keys::bonding_amount_key(user);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);
}

pub fn unbond(user: PublicKey, maybe_amount: Option<U512>) -> Result<U512> {
    let key = local_keys::bonding_amount_key(user);
    let bonding_amount = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();

    let unbond_amount = match maybe_amount {
        Some(amount) => amount,
        None => bonding_amount,
    };

    // validate amount
    {
        if unbond_amount > bonding_amount {
            return Err(Error::UnbondTooLarge);
        }

        // TODO: make iteration to make sure not to ommit an action.
        let max_action_amount = U512::max(read_delegating_amount(user), read_voting_amount(user));
        if unbond_amount > bonding_amount - max_action_amount {
            return Err(Error::UnbondTooLarge);
        }
    }

    storage::write_local(key, bonding_amount - unbond_amount);
    Ok(unbond_amount)
}

pub fn read_delegation(delegator: PublicKey, validator: PublicKey) -> U512 {
    let key = local_keys::delegation_key(delegator, validator);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_delegating_amount(delegator: PublicKey) -> U512 {
    let key = local_keys::delegating_amount_key(delegator);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_delegated_amount(validator: PublicKey) -> U512 {
    let key = local_keys::delegated_amount_key(validator);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn delegate(delegator: PublicKey, validator: PublicKey, amount: U512) -> Result<()> {
    // update delegating amount (delegator, amount)
    let delegating_amount_key = local_keys::delegating_amount_key(delegator);
    let delegating_amount: U512 = storage::read_local(&delegating_amount_key)
        .unwrap_or_default()
        .unwrap_or_default();

    // validate amount
    {
        let bonding_amount = read_bonding_amount(delegator);
        let delegating_amount = read_delegating_amount(delegator);
        // internal error
        if delegating_amount > bonding_amount {
            // TODO: return Err(Error::InternalError);
            return Err(Error::NotBonded);
        }
        if amount > bonding_amount - delegating_amount {
            // TODO: return Err(Error::DelegateMoreThanStakes);
            return Err(Error::UndelegateTooLarge);
        }
    }
    storage::write_local(delegating_amount_key, delegating_amount + amount);

    // update delegation ((delegator, validator), amount)
    let key = local_keys::delegation_key(delegator, validator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    // update delegated amount (validator, amount)
    let key = local_keys::delegated_amount_key(validator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    Ok(())
}

pub fn undelegate(
    delegator: PublicKey,
    validator: PublicKey,
    maybe_amount: Option<U512>,
) -> Result<U512> {
    // update delegation ((delegator, validator), amount)
    let delegation_key = local_keys::delegation_key(delegator, validator);
    let delegation_amount: U512 = storage::read_local(&delegation_key)
        .unwrap_or_default()
        .unwrap_or_default();
    let undelegate_amount = match maybe_amount {
        Some(amount) => {
            if amount > delegation_amount {
                return Err(Error::UndelegateTooLarge);
            }
            if amount.is_zero() {
                // TODO: change to UndelegateTooSmall;
                return Err(Error::UndelegateTooLarge);
            }
            amount
        }
        None => delegation_amount,
    };
    storage::write_local(delegation_key, delegation_amount - undelegate_amount);

    // update delegating amount (delegator, amount)
    let key = local_keys::delegating_amount_key(delegator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - undelegate_amount);

    // update delegated amount (validator, amount)
    let key = local_keys::delegated_amount_key(validator);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - undelegate_amount);
    Ok(undelegate_amount)
}

pub fn redelegate(
    delegator: PublicKey,
    src_validator: PublicKey,
    dest_validator: PublicKey,
    maybe_amount: Option<U512>,
) -> Result<()> {
    // update delegation(delegator, src_validator)
    let src_delegation_key = local_keys::delegation_key(delegator, src_validator);
    let delegation_amount: U512 = storage::read_local(&src_delegation_key)
        .unwrap_or_default()
        .unwrap_or_default();

    let redelegate_amount = match maybe_amount {
        Some(amount) => {
            if amount > delegation_amount {
                return Err(Error::UndelegateTooLarge);
            }
            if amount.is_zero() {
                // TODO: UndelegateTooSmall
                return Err(Error::UndelegateTooLarge);
            }
            amount
        }
        None => delegation_amount,
    };

    storage::write_local(src_delegation_key, delegation_amount - redelegate_amount);

    // update delegation(delegator, dest_validator)
    let dest_delegation_key = local_keys::delegation_key(delegator, dest_validator);
    let delegation_amount: U512 = storage::read_local(&dest_delegation_key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(dest_delegation_key, delegation_amount + redelegate_amount);

    Ok(())
}

pub fn read_vote(voter: PublicKey, dapp: Key) -> U512 {
    let key = local_keys::vote_key(voter, dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voting_amount(voter: PublicKey) -> U512 {
    let key = local_keys::voting_amount_key(voter);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn read_voted_amount(dapp: Key) -> U512 {
    let key = local_keys::voted_amount_key(dapp);
    storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default()
}

pub fn vote(voter: PublicKey, dapp: Key, amount: U512) -> Result<()> {
    // update voting amount (voter, amount)
    let voting_amount_key = local_keys::voting_amount_key(voter);
    let voting_amount: U512 = storage::read_local(&voting_amount_key)
        .unwrap_or_default()
        .unwrap_or_default();

    // validate amount
    {
        if amount.is_zero() {
            // TODO: change to Error::VoteTooSmall
            return Err(Error::BondTooSmall);
        }

        let bonding_amount = read_bonding_amount(voter);

        if voting_amount > bonding_amount {
            // TODO: Internal Error
            return Err(Error::VoteTooLarge);
        }
        if amount > bonding_amount - voting_amount {
            return Err(Error::VoteTooLarge);
        }
    }
    storage::write_local(voting_amount_key, voting_amount + amount);

    // update vote ((voter, dapp), amount)
    let key = local_keys::vote_key(voter, dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    // update voted amount (dapp, amount)
    let key = local_keys::voted_amount_key(dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    storage::write_local(key, current_amount + amount);

    Ok(())
}

pub fn unvote(voter: PublicKey, dapp: Key, amount: U512) -> Result<()> {
    // update vote ((voter, dapp), amount)
    let vote_key = local_keys::vote_key(voter, dapp);
    let vote_amount = storage::read_local(&vote_key)
        .unwrap_or_default()
        .unwrap_or_default();
    if amount > vote_amount {
        return Err(Error::UnvoteTooLarge);
    }
    storage::write_local(vote_key, vote_amount - amount);

    // update voting amount (voter, amount)
    let key = local_keys::voting_amount_key(voter);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);

    // update voted amount (dapp, amount)
    let key = local_keys::voted_amount_key(dapp);
    let current_amount: U512 = storage::read_local(&key)
        .unwrap_or_default()
        .unwrap_or_default();
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    storage::write_local(key, current_amount - amount);
    Ok(())
}
