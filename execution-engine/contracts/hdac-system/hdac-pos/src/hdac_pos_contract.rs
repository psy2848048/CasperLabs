use alloc::collections::BTreeMap;

use contract::contract_api::storage;
use proof_of_stake::{MintProvider, ProofOfStake, RuntimeProvider, Stakes, StakesProvider};
use types::{
    account::{PublicKey, PurseId},
    system_contract_errors::pos::{Error, PurseLookupError, Result},
    Key, URef, U512,
};

use crate::{
    constants::{local_keys, uref_names},
    contract_delegations::DelegationKey,
    contract_mint::ContractMint,
    contract_queue::ContractQueue,
    contract_runtime::ContractRuntime,
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

        // increase validator's staked token amount
        let mut stakes: Stakes = ContractStakes::read()?;
        stakes.bond(&validator, amount);
        ContractStakes::write(&stakes);

        // update delegation table
        let del_key = DelegationKey {
            delegator,
            validator,
        };
        let mut delegations: BTreeMap<DelegationKey, U512> =
            storage::read_local::<_, _>(&local_keys::DELEGATION_MAP_KEY)
                .unwrap_or_default()
                .unwrap_or_default();

        delegations
            .entry(del_key)
            .and_modify(|x| *x += amount)
            .or_insert(amount);

        // write updated delegation.
        storage::write_local::<_, _>(del_key, delegations);
        Ok(())
    }

    pub fn undelegate(
        &self,
        _delegator: PublicKey,
        _validator: PublicKey,
        _shares: U512,
    ) -> Result<()> {
        Ok(())
    }

    pub fn redelegate(
        &self,
        _delegator: PublicKey,
        _src: PublicKey,
        _dest: PublicKey,
        _shares: U512,
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
