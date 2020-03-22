use contract::unwrap_or_revert::UnwrapOrRevert;
use proof_of_stake::{MintProvider, ProofOfStake, RuntimeProvider, Stakes, StakesProvider};
use types::{
    account::{PublicKey, PurseId},
    system_contract_errors::pos::{Error, PurseLookupError, Result},
    Key, URef, U512,
};

use crate::{
    constants::uref_names,
    contract_delegations::ContractDelegations,
    contract_mint::ContractMint,
    contract_queue::ContractQueue,
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

        // TODO: enqueue a new item and dequeue items to process

        // increase validator's staked token amount
        let mut stakes: Stakes = ContractStakes::read()?;

        // if this is not self-delegation and target validator is not bonded
        if delegator != validator && !stakes.0.contains_key(&validator) {
            return Err(Error::NotBonded);
        }

        stakes.bond(&validator, amount);
        ContractStakes::write(&stakes);

        // update delegation table.
        let mut delegations = ContractDelegations::read()?;
        delegations.delegate(&delegator, &validator, amount);
        ContractDelegations::write(&delegations);

        Ok(())
    }

    pub fn undelegate(
        &self,
        delegator: PublicKey,
        validator: PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()> {
        // TODO: enqueue a new item and dequeue items to process
        let mut delegations = ContractDelegations::read()?;
        let amount = delegations.undelegate(&delegator, &validator, maybe_amount)?;
        ContractDelegations::write(&delegations);

        let mut stakes = ContractStakes::read()?;
        let payout = stakes.unbond(&validator, Some(amount))?;
        ContractStakes::write(&stakes);

        let pos_purse = get_purse_id::<ContractRuntime>(uref_names::POS_BONDING_PURSE)
            .map_err(PurseLookupError::bonding)?;

        ContractMint::transfer_from_purse_to_account(pos_purse, delegator, payout)
            .map_err(|_| Error::UnbondTransferFailed)?;
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

        let mut delegations = ContractDelegations::read()?;
        let amount = delegations.undelegate(&delegator, &src, Some(amount))?;
        delegations.delegate(&delegator, &dest, amount);
        ContractDelegations::write(&delegations);

        let mut stakes = ContractStakes::read()?;
        let payout = stakes.unbond(&src, Some(amount))?;
        stakes.bond(&dest, payout);
        ContractStakes::write(&stakes);

        Ok(())
    }

    pub fn vote(&self, user: PublicKey, dapp: Key, amount: U512) -> Result<()> {
        // staked balance check
        if amount.is_zero() {
            return Err(Error::BondTooSmall);
        }

        // check validator's staked token amount
        let stakes: Stakes = ContractStakes::read()?;
        // if an user has no staked amount, he cannot do anything
        let staked_balance: U512 = *stakes
            .0
            .get(&user)
            .unwrap_or_revert_with(Error::StakesNotFound);

        // check user's vote stat
        let vote_stat: VoteStat = ContractVotes::read_stat()?;
        let zero_const = U512::from(0);
        let vote_stat_per_user: U512 = *vote_stat.0.get(&user).unwrap_or_else(|| &zero_const);

        if staked_balance < vote_stat_per_user + amount {
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
