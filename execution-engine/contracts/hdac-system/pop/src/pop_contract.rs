<<<<<<< HEAD
=======
use contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
>>>>>>> chore: obeyed the linter majesty
use proof_of_stake::{MintProvider, ProofOfStake, RuntimeProvider, Stakes, StakesProvider};
use types::{
    account::{PublicKey, PurseId},
    system_contract_errors::pos::{Error, PurseLookupError, Result},
    BlockTime, Key, URef, U512,
};

use crate::{
    constants::{local_keys, uref_names},
    contract_delegations::ContractDelegations,
    contract_mint::ContractMint,
    contract_queue::{
        ContractQueue, DelegateRequestKey, RedelegateRequestKey, UndelegateRequestKey,
    },
    contract_runtime::ContractRuntime,
    contract_stakes::ContractStakes,
    contract_votes::{ContractVotes, VoteStat, Votes},
};

pub struct ProofOfProfessionContract;

impl ProofOfStake<ContractMint, ContractQueue, ContractRuntime, ContractStakes>
    for ProofOfProfessionContract
{
    fn bond(&self, _: PublicKey, _: U512, _: URef) -> Result<()> {
        Err(Error::NotSupportedFunc)
    }

    fn unbond(&self, _: PublicKey, _: Option<U512>) -> Result<()> {
        Err(Error::NotSupportedFunc)
    }

    fn step(&self) -> Result<()> {
        Err(Error::NotSupportedFunc)
    }
}

impl ProofOfProfessionContract {
    pub fn delegate(
        &self,
        delegator: PublicKey,
        validator: PublicKey,
        amount: U512,
        source_purse: URef,
    ) -> Result<()> {
        // transfer amount to pos_bonding_purse
        if amount.is_zero() {
            return Err(Error::BondTooSmall);
        }
        let source = PurseId::new(source_purse);
        let pos_purse = get_purse_id::<ContractRuntime>(uref_names::POS_BONDING_PURSE)
            .map_err(PurseLookupError::bonding)?;

        ContractMint::transfer_from_purse_to_purse(source, pos_purse, amount)
            .map_err(|_| Error::BondTransferFailed)?;

        // check validator is bonded
        let stakes: Stakes = ContractStakes::read()?;
        // if this is not self-delegation and target validator is not bonded
        if delegator != validator && !stakes.0.contains_key(&validator) {
            return Err(Error::NotBonded);
        }

        let mut request_queue =
            ContractQueue::read_requests::<DelegateRequestKey>(local_keys::DELEGATE_REQUEST_QUEUE);

        request_queue.push(
            DelegateRequestKey::new(delegator, validator),
            amount,
            ContractRuntime::get_block_time(),
        )?;

        ContractQueue::write_requests(local_keys::DELEGATE_REQUEST_QUEUE, request_queue);

        // TODO: this should be factored out to ProofOfStake::step.
        Self::step_delegation(ContractRuntime::get_block_time())?;
        Ok(())
    }

    fn step_delegation(timestamp: BlockTime) -> Result<()> {
        let mut request_queue =
            ContractQueue::read_requests::<DelegateRequestKey>(local_keys::DELEGATE_REQUEST_QUEUE);
        let requests = request_queue.pop_due(timestamp);

        let mut stakes: Stakes = ContractStakes::read()?;
        let mut delegations = ContractDelegations::read()?;

        for request in requests {
            let DelegateRequestKey {
                delegator,
                validator,
            } = request.request_key;

            stakes.bond(&validator, request.amount);
            delegations.delegate(&delegator, &validator, request.amount);
        }

        ContractStakes::write(&stakes);
        ContractDelegations::write(&delegations);

        ContractQueue::write_requests(local_keys::DELEGATE_REQUEST_QUEUE, request_queue);
        Ok(())
    }

    pub fn undelegate(
        &self,
        delegator: PublicKey,
        validator: PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()> {
        let mut request_queue = ContractQueue::read_requests::<UndelegateRequestKey>(
            local_keys::UNDELEGATE_REQUEST_QUEUE,
        );

        let amount = match maybe_amount {
            None => U512::from(0),
            Some(amount) => amount,
        };

        request_queue.push(
            UndelegateRequestKey::new(delegator, validator),
            amount,
            ContractRuntime::get_block_time(),
        )?;

        ContractQueue::write_requests(local_keys::UNDELEGATE_REQUEST_QUEUE, request_queue);

        // TODO: this should be factored out to ProofOfStake::step.
        Self::step_undelegation(ContractRuntime::get_block_time())?;
        Ok(())
    }

    fn step_undelegation(timestamp: BlockTime) -> Result<()> {
        let mut request_queue = ContractQueue::read_requests::<UndelegateRequestKey>(
            local_keys::UNDELEGATE_REQUEST_QUEUE,
        );
        let requests = request_queue.pop_due(timestamp);

        let mut delegations = ContractDelegations::read()?;
        let mut stakes = ContractStakes::read()?;
        let pos_purse = get_purse_id::<ContractRuntime>(uref_names::POS_BONDING_PURSE)
            .map_err(PurseLookupError::bonding)?;

        for request in requests {
            let UndelegateRequestKey {
                delegator,
                validator,
            } = request.request_key;

            let maybe_amount = match request.amount {
                val if val == U512::from(0) => None,
                _ => Some(request.amount),
            };

            let amount = delegations.undelegate(&delegator, &validator, maybe_amount)?;
            let payout = stakes.unbond(&validator, Some(amount))?;
            ContractMint::transfer_from_purse_to_account(pos_purse, delegator, payout)
                .map_err(|_| Error::UnbondTransferFailed)?;
        }

        ContractDelegations::write(&delegations);
        ContractStakes::write(&stakes);
        ContractQueue::write_requests(local_keys::UNDELEGATE_REQUEST_QUEUE, request_queue);
        Ok(())
    }

    pub fn redelegate(
        &self,
        delegator: PublicKey,
        src: PublicKey,
        dest: PublicKey,
        amount: U512,
    ) -> Result<()> {
        if src == dest {
            return Err(Error::SelfRedelegation);
        }

        let mut request_queue = ContractQueue::read_requests::<RedelegateRequestKey>(
            local_keys::REDELEGATE_REQUEST_QUEUE,
        );

        request_queue.push(
            RedelegateRequestKey::new(delegator, src, dest),
            amount,
            ContractRuntime::get_block_time(),
        )?;

        ContractQueue::write_requests(local_keys::REDELEGATE_REQUEST_QUEUE, request_queue);

        // TODO: this should be factored out to ProofOfStake::step.
        Self::step_redelegation(ContractRuntime::get_block_time())?;

        Ok(())
    }

    fn step_redelegation(timestamp: BlockTime) -> Result<()> {
        let mut request_queue = ContractQueue::read_requests::<RedelegateRequestKey>(
            local_keys::REDELEGATE_REQUEST_QUEUE,
        );

        let requests = request_queue.pop_due(timestamp);
        let mut delegations = ContractDelegations::read()?;
        let mut stakes = ContractStakes::read()?;

        for request in requests {
            let RedelegateRequestKey {
                delegator,
                src_validator,
                dest_validator,
            } = request.request_key;

            let amount =
                delegations.undelegate(&delegator, &src_validator, Some(request.amount))?;
            delegations.delegate(&delegator, &dest_validator, amount);
            let payout = stakes.unbond(&src_validator, Some(amount))?;
            stakes.bond(&dest_validator, payout);
        }

        ContractDelegations::write(&delegations);
        ContractStakes::write(&stakes);
        ContractQueue::write_requests(local_keys::REDELEGATE_REQUEST_QUEUE, request_queue);
        Ok(())
    }

    pub fn vote(&self, user: PublicKey, dapp: Key, amount: U512) -> Result<()> {
        // staked balance check
        if amount.is_zero() {
            return Err(Error::BondTooSmall);
        }

        // check validator's staked token amount
        let delegation_user_stat = ContractDelegations::read_user_stat()?;
        // if an user has no staked amount, he cannot do anything
        let delegated_balance: U512 = match delegation_user_stat.0.get(&user) {
            Some(balance) => *balance,
            None => return Err(Error::DelegationsNotFound),
        };

        // check user's vote stat
        let vote_stat: VoteStat = ContractVotes::read_stat()?;
        let vote_stat_per_user: U512 = vote_stat
            .0
            .get(&user)
            .cloned()
            .unwrap_or_else(|| U512::from(0));

        if delegated_balance < vote_stat_per_user + amount {
            return Err(Error::VoteTooLarge);
        }

        // check vote table
        let mut votes: Votes = ContractVotes::read()?; // <- here
        votes.vote(&user, &dapp, amount);
        ContractVotes::write(&votes);

        Ok(())
    }

    pub fn unvote(&self, user: PublicKey, dapp: Key, maybe_amount: Option<U512>) -> Result<()> {
        let mut votes = ContractVotes::read()?;
        votes.unvote(&user, &dapp, maybe_amount)?;
        ContractVotes::write(&votes);

        Ok(())
    }
}

fn get_purse_id<R: RuntimeProvider>(name: &str) -> core::result::Result<PurseId, PurseLookupError> {
    R::get_key(name)
        .ok_or(PurseLookupError::KeyNotFound)
        .and_then(|key| match key {
            Key::URef(uref) => Ok(PurseId::new(uref)),
            _ => Err(PurseLookupError::KeyUnexpectedType),
        })
}
