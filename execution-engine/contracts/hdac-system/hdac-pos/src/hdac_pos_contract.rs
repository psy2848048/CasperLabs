use proof_of_stake::ProofOfStake;
use types::{account::PublicKey, system_contract_errors::pos::Result, U512};

use crate::{
    contract_mint::ContractMint, contract_queue::ContractQueue, contract_runtime::ContractRuntime,
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
        _delegator: PublicKey,
        _validator: PublicKey,
        _amount: U512,
    ) -> Result<()> {
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
