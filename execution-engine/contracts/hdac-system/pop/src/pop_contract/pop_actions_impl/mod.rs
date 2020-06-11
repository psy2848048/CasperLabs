mod delegation;
pub mod stake;
mod vote;

pub use delegation::{DelegationKey, Delegations};

use contract::contract_api::{runtime, system};
use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, PurseLookupError, Result},
    Key, URef, U512,
};

use super::{
    pop_actions::{Delegatable, ProofOfProfession, Stakable, Votable},
    ProofOfProfessionContract,
};
use crate::{
    constants::uref_names,
    store::{self, RedelegateRequest, UnbondRequest, UndelegateRequest},
};

impl ProofOfProfession for ProofOfProfessionContract {}

impl Stakable for ProofOfProfessionContract {
    fn bond(&mut self, user: PublicKey, amount: U512, source_purse: URef) -> Result<()> {
        // transfer amount to pos_bonding_purse
        if amount.is_zero() {
            return Err(Error::BondTooSmall);
        }
        let pos_purse =
            get_purse(uref_names::POS_BONDING_PURSE).map_err(PurseLookupError::bonding)?;

        system::transfer_from_purse_to_purse(source_purse, pos_purse, amount)
            .map_err(|_| Error::BondTransferFailed)?;

        // write own staking amount
        stake::bond(&user, amount);

        Ok(())
    }

    fn unbond(&mut self, requester: PublicKey, maybe_amount: Option<U512>) -> Result<()> {
        // validate unbond amount
        if let Some(amount) = maybe_amount {
            let current_amount = store::read_bonding_amount(&requester);

            // The over-amount caused by the accumulated unbonding request amount is handled in
            // step phase
            if amount > current_amount {
                return Err(Error::UnbondTooLarge);
            }
        }

        // write unbond request
        let current = runtime::get_blocktime();
        let mut queue = store::read_unbond_requests();
        queue.push(
            UnbondRequest {
                requester,
                maybe_amount,
            },
            current,
        )?;
        store::write_unbond_requests(queue);

        Ok(())
    }
}

impl Delegatable for ProofOfProfessionContract {
    fn delegate(&mut self, delegator: PublicKey, validator: PublicKey, amount: U512) -> Result<()> {
        let mut delegations = store::read_delegations()?;
        delegations.delegate(&delegator, &validator, amount)?;
        store::write_delegations(&delegations);
        Ok(())
    }

    fn undelegate(
        &mut self,
        delegator: PublicKey,
        validator: PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()> {
        // validate undelegate amount
        if let Some(amount) = maybe_amount {
            let delegation_amount =
                store::read_delegations()?.delegation(&delegator, &validator)?;

            // The over-amount caused by the accumulated undelegating request amount is handled
            // step phase
            if amount > delegation_amount {
                return Err(Error::UndelegateTooLarge);
            }
        }

        let mut queue = store::read_undelegation_requests();
        queue.push(
            UndelegateRequest {
                delegator,
                validator,
                maybe_amount,
            },
            runtime::get_blocktime(),
        )?;
        store::write_undelegation_requests(queue);

        Ok(())
    }

    fn redelegate(
        &mut self,
        delegator: PublicKey,
        src: PublicKey,
        dest: PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()> {
        if src == dest {
            return Err(Error::SelfRedelegation);
        }

        // // validate redelegate amount
        if let Some(amount) = maybe_amount {
            let delegation_amount = store::read_delegations()?.delegation(&delegator, &src)?;

            // The over-amount caused by the accumulated undelegating request amount is handled
            // step phase
            if amount > delegation_amount {
                return Err(Error::UndelegateTooLarge);
            }
        }

        let mut request_queue = store::read_redelegation_requests();
        request_queue.push(
            RedelegateRequest {
                delegator: delegator,
                src_validator: src,
                dest_validator: dest,
                maybe_amount: maybe_amount,
            },
            runtime::get_blocktime(),
        )?;
        store::write_redelegation_requests(request_queue);

        Ok(())
    }
}

impl Votable for ProofOfProfessionContract {
    fn vote(&mut self, user: PublicKey, dapp: Key, amount: U512) -> Result<()> {
        vote::vote(&user, &dapp, amount)?;
        Ok(())
    }

    fn unvote(&mut self, user: PublicKey, dapp: Key, maybe_amount: Option<U512>) -> Result<()> {
        let vote = store::read_vote(&user, &dapp);
        let unvote_amount = match maybe_amount {
            Some(amount) => {
                if amount > vote {
                    return Err(Error::UnvoteTooLarge);
                }
                amount
            }
            None => vote,
        };

        vote::unvote(&user, &dapp, unvote_amount)?;
        Ok(())
    }
}

fn get_purse(name: &str) -> core::result::Result<URef, PurseLookupError> {
    runtime::get_key(name)
        .ok_or(PurseLookupError::KeyNotFound)
        .and_then(|key| match key {
            Key::URef(uref) => Ok(uref),
            _ => Err(PurseLookupError::KeyUnexpectedType),
        })
}
