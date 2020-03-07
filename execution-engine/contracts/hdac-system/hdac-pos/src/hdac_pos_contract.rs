use alloc::{collections::BTreeMap, vec::Vec};
use core::result;

use contract::contract_api::storage;
use proof_of_stake::ProofOfStake;
use types::{
    account::PublicKey,
    bytesrepr::{self, FromBytes, ToBytes},
    system_contract_errors::pos::Result,
    CLType, CLTyped, U512,
};

use crate::{
    constants::local_keys::DELEGATION_MAP_KEY, contract_mint::ContractMint,
    contract_queue::ContractQueue, contract_runtime::ContractRuntime,
    contract_stakes::ContractStakes,
};

pub struct DelegatedProofOfStakeContract;

impl ProofOfStake<ContractMint, ContractQueue, ContractRuntime, ContractStakes>
    for DelegatedProofOfStakeContract
{
    // fn bond(&self, validator: PublicKey, amount: U512, source_uref: URef) -> Result<()>;
    // fn unbond(&self, validator: PublicKey, maybe_amount: Option<U512>) -> Result<()>;
    // fn step(&self) -> Result<()>;
    // fn get_payment_purse(&self) -> Result<PurseId>;
    // fn set_refund_purse(&self, purse_id: PurseId) -> Result<()>;
    // fn get_refund_purse(&self) -> Result<Option<PurseId>>;
    // fn finalize_payment(&self, amount_spent: U512, account: PublicKey) -> Result<()>;
}

impl DelegatedProofOfStakeContract {
    pub fn delegate(
        &self,
        _delegator: PublicKey,
        _validator: PublicKey,
        _amount: U512,
    ) -> Result<()> {
        // Get or Create Delegation
        let _delegations =
            storage::read_local::<_, BTreeMap<DelegationKey, U512>>(&DELEGATION_MAP_KEY)
                .unwrap_or_default()
                .unwrap_or_default();
        // transfer amount to pos_bonding_purse
        // Get Stakes of validators
        // increase validator's staked token amount and calculate shares.
        // update delegation's share.
        // write_local DELEGATION_MAP with updated delegation.
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

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
struct DelegationKey {
    delegator: PublicKey,
    validator: PublicKey,
}

impl FromBytes for DelegationKey {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (delegator, bytes) = PublicKey::from_bytes(bytes)?;
        let (validator, bytes) = PublicKey::from_bytes(bytes)?;
        let entry = DelegationKey {
            delegator,
            validator,
        };
        Ok((entry, bytes))
    }
}

impl ToBytes for DelegationKey {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        Ok((self.delegator.to_bytes()?.into_iter())
            .chain(self.validator.to_bytes()?)
            .collect())
    }
}

impl CLTyped for DelegationKey {
    fn cl_type() -> CLType {
        CLType::Any
    }
}
