use contract::contract_api::{runtime, storage, system};
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
    constants::{local_keys, uref_names},
    local_store::{self, RedelegateRequest, UnbondRequest, UndelegateRequest},
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
        let key = local_keys::staking_amount_key(user);
        let current_amount: U512 = storage::read_local(&key)
            .unwrap_or_default()
            .unwrap_or_default();
        storage::write_local(key, current_amount + amount);

        Ok(())
    }

    fn unbond(&mut self, requester: PublicKey, maybe_amount: Option<U512>) -> Result<()> {
        // validating request
        let key = local_keys::staking_amount_key(requester);
        let current_amount: U512 = storage::read_local(&key)
            .unwrap_or_default()
            .unwrap_or_default();

        if let Some(amount) = maybe_amount {
            if current_amount < amount {
                return Err(Error::UnbondTooLarge);
            }
        }

        // write unbond request
        let current = runtime::get_blocktime();
        let mut queue = local_store::read_unbond_requests();
        queue.push(
            UnbondRequest {
                requester,
                maybe_amount,
            },
            current,
        )?;
        local_store::write_unbond_requests(queue);

        Ok(())
    }
}

impl Delegatable for ProofOfProfessionContract {
    fn delegate(&mut self, delegator: PublicKey, validator: PublicKey, amount: U512) -> Result<()> {
        // TODO: validate validator is created.

        let staking_amount: U512 = storage::read_local(&local_keys::staking_amount_key(delegator))
            .unwrap_or_default()
            .unwrap_or_default();
        let delegating_amount: U512 =
            storage::read_local(&local_keys::delegating_amount_key(delegator))
                .unwrap_or_default()
                .unwrap_or_default();

        // internal error
        if staking_amount < delegating_amount {
            // TODO: return Err(Error::InternalError);
            return Err(Error::NotBonded);
        }
        if staking_amount - delegating_amount < amount {
            // TODO: return Err(Error::DelegateMoreThanStakes);
            return Err(Error::UndelegateTooLarge);
        }

        // write delegation
        storage::write_local(local_keys::delegation_key(delegator, validator), amount);

        // write delegating amount
        storage::write_local(
            local_keys::delegating_amount_key(delegator),
            delegating_amount + amount,
        );

        // write delegated amount
        storage::write_local(local_keys::delegated_amount_key(validator), amount);

        // TODO: update named_key
        Ok(())
    }

    fn undelegate(
        &mut self,
        delegator: PublicKey,
        validator: PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()> {
        // validate undelegation by simulating
        let mut delegations = self.read_delegations()?;
        let mut stakes = self.read_stakes()?;
        let amount = delegations.undelegate(&delegator, &validator, maybe_amount)?;
        let _ = stakes.unbond(&validator, Some(amount))?;

        let mut request_queue = local_store::read_undelegation_requests();

        request_queue.push(
            UndelegateRequest {
                delegator,
                validator,
                maybe_amount,
            },
            runtime::get_blocktime(),
        )?;

        local_store::write_undelegation_requests(request_queue);
        Ok(())
    }

    fn redelegate(
        &mut self,
        delegator: PublicKey,
        src: PublicKey,
        dest: PublicKey,
        amount: U512,
    ) -> Result<()> {
        if src == dest {
            return Err(Error::SelfRedelegation);
        }
        // validate redelegation by simulating
        let mut delegations = self.read_delegations()?;
        let mut stakes = self.read_stakes()?;
        let amount = delegations.undelegate(&delegator, &src, Some(amount))?;
        let _payout = stakes.unbond(&src, Some(amount))?;

        let mut request_queue = local_store::read_redelegation_requests();

        request_queue.push(
            RedelegateRequest {
                delegator: delegator,
                src_validator: src,
                dest_validator: dest,
                maybe_amount: Some(amount),
            },
            runtime::get_blocktime(),
        )?;

        local_store::write_redelegation_requests(request_queue);
        Ok(())
    }
}

impl Votable for ProofOfProfessionContract {
    fn vote(&mut self, user: PublicKey, dapp: Key, amount: U512) -> Result<()> {
        // staked balance check
        if amount.is_zero() {
            return Err(Error::BondTooSmall);
        }

        // check validator's staked token amount
        let delegation_user_stat = self.read_delegation_user_stat()?;
        // if an user has no staked amount, he cannot do anything
        let delegated_balance: U512 = match delegation_user_stat.0.get(&user) {
            Some(balance) => *balance,
            None => return Err(Error::DelegationsNotFound),
        };

        // check user's vote stat
        let vote_stat = self.read_vote_stat()?;
        let vote_stat_per_user: U512 = vote_stat
            .0
            .get(&user)
            .cloned()
            .unwrap_or_else(|| U512::from(0));

        if delegated_balance < vote_stat_per_user + amount {
            return Err(Error::VoteTooLarge);
        }

        // check vote table
        let mut votes = self.read_votes()?; // <- here
        votes.vote(&user, &dapp, amount);
        self.write_votes(&votes);

        Ok(())
    }

    fn unvote(&mut self, user: PublicKey, dapp: Key, maybe_amount: Option<U512>) -> Result<()> {
        let mut votes = self.read_votes()?;
        votes.unvote(&user, &dapp, maybe_amount)?;
        self.write_votes(&votes);

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
