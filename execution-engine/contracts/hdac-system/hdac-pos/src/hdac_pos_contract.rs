use proof_of_stake::{MintProvider, ProofOfStake, RuntimeProvider, Stakes, StakesProvider};
use types::{
    account::{PublicKey, PurseId},
    system_contract_errors::pos::{Error, PurseLookupError, Result},
    Key, URef, U512,
};

use crate::{
    constants::uref_names, contract_delegations::ContractDelegations, contract_mint::ContractMint,
    contract_queue::ContractQueue, contract_runtime::ContractRuntime,
    contract_stakes::ContractStakes,
};

pub struct DelegatedProofOfStakeContract;

impl ProofOfStake<ContractMint, ContractQueue, ContractRuntime, ContractStakes>
    for DelegatedProofOfStakeContract
{
}

impl DelegatedProofOfStakeContract {
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
        _delegator: PublicKey,
        _src: PublicKey,
        _dest: PublicKey,
        _amount: U512,
    ) -> Result<()> {
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
