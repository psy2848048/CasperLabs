mod delegations;
mod economy;
mod request_pool;
mod stakes_provider;
mod votes;

use alloc::collections::BTreeMap;
use contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use proof_of_stake::{
    MintProvider, ProofOfStake, Queue, QueueProvider, RuntimeProvider, Stakes, StakesProvider,
};
use types::{
    account::PublicKey,
    system_contract_errors::{
        mint,
        pos::{Error, PurseLookupError, Result},
    },
    BlockTime, Key, Phase, TransferResult, URef, U512,
};

use crate::constants::{consts, uref_names};

use delegations::ContractDelegations;
use economy::{pop_score_calculation, ContractClaim};
use request_pool::{
    ClaimRequest, ContractQueue, DelegateRequestKey, DelegationKind, RedelegateRequestKey,
    UndelegateRequestKey,
};
use votes::{ContractVotes, VoteStat, Votes};

pub struct ProofOfProfessionContract;

impl StakesProvider for ProofOfProfessionContract {
    fn read(&self) -> Result<Stakes> {
        self.read_stakes()
    }

    fn write(&mut self, stakes: &Stakes) {
        self.write_stakes(stakes);
    }
}
impl QueueProvider for ProofOfProfessionContract {
    // TODO: remove QueueProvider
    // Currently, we are utilizing the default implemention of the Proof-of-Stake crate,
    // so we need to add a dummy implemention to meet trait contraint.

    fn read_bonding(&mut self) -> Queue {
        unimplemented!()
    }
    fn read_unbonding(&mut self) -> Queue {
        unimplemented!()
    }
    fn write_bonding(&mut self, _: Queue) {
        unimplemented!()
    }
    fn write_unbonding(&mut self, _: Queue) {
        unimplemented!()
    }
}
impl RuntimeProvider for ProofOfProfessionContract {
    fn get_key(&self, name: &str) -> Option<Key> {
        runtime::get_key(name)
    }

    fn put_key(&mut self, name: &str, key: Key) {
        runtime::put_key(name, key)
    }

    fn remove_key(&mut self, name: &str) {
        runtime::remove_key(name)
    }

    fn get_phase(&self) -> Phase {
        runtime::get_phase()
    }

    fn get_block_time(&self) -> BlockTime {
        runtime::get_blocktime()
    }

    fn get_caller(&self) -> PublicKey {
        runtime::get_caller()
    }
}
impl MintProvider for ProofOfProfessionContract {
    fn transfer_purse_to_account(
        &mut self,
        source: URef,
        target: PublicKey,
        amount: U512,
    ) -> TransferResult {
        system::transfer_from_purse_to_account(source, target, amount)
    }

    fn transfer_purse_to_purse(
        &mut self,
        source: URef,
        target: URef,
        amount: U512,
    ) -> core::result::Result<(), ()> {
        system::transfer_from_purse_to_purse(source, target, amount).map_err(|_| ())
    }

    fn balance(&mut self, purse: URef) -> Option<U512> {
        system::get_balance(purse)
    }
}

impl ProofOfStake for ProofOfProfessionContract {
    fn bond(&mut self, _: PublicKey, _: U512, _: URef) -> Result<()> {
        Err(Error::NotSupportedFunc)
    }

    fn unbond(&mut self, _: PublicKey, _: Option<U512>) -> Result<()> {
        Err(Error::NotSupportedFunc)
    }
}

impl ProofOfProfessionContract {
    pub fn step(&self) -> Result<()> {
        // let blocktime = runtime::get_blocktime();
        // self.step_undelegation(blocktime);
        // self.step_redelegation(blocktime);
        let caller = runtime::get_caller();

        if caller.value() != consts::SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        self.distribute()?;
        self.step_claim()?;

        Ok(())
    }

    pub fn delegate(
        &mut self,
        delegator: PublicKey,
        validator: PublicKey,
        amount: U512,
        source_purse: URef,
    ) -> Result<()> {
        // transfer amount to pos_bonding_purse
        if amount.is_zero() {
            return Err(Error::BondTooSmall);
        }
        let pos_purse =
            get_purse(self, uref_names::POS_BONDING_PURSE).map_err(PurseLookupError::bonding)?;

        self.transfer_purse_to_purse(source_purse, pos_purse, amount)
            .map_err(|_| Error::BondTransferFailed)?;

        // check validator is bonded
        let stakes = self.read()?;
        // if this is not self-delegation and target validator is not bonded
        if delegator != validator && !stakes.0.contains_key(&validator) {
            return Err(Error::NotBonded);
        }

        let mut request_queue =
            ContractQueue::read_delegation_requests::<DelegateRequestKey>(DelegationKind::Delegate);

        request_queue.push(
            DelegateRequestKey::new(delegator, validator),
            amount,
            self.get_block_time(),
        )?;

        ContractQueue::write_delegation_requests(DelegationKind::Delegate, request_queue);

        // TODO: this should be factored out to ProofOfStake::step.
        self.step_delegation(self.get_block_time())?;
        Ok(())
    }

    fn step_delegation(&mut self, timestamp: BlockTime) -> Result<()> {
        let mut request_queue =
            ContractQueue::read_delegation_requests::<DelegateRequestKey>(DelegationKind::Delegate);
        let requests = request_queue.pop_due(timestamp);

        let mut stakes: Stakes = self.read()?;
        let mut delegations = ContractDelegations::read()?;

        for request in requests {
            let DelegateRequestKey {
                delegator,
                validator,
            } = request.request_key;

            stakes.bond(&validator, request.amount);
            delegations.delegate(&delegator, &validator, request.amount);
        }

        self.write(&stakes);
        ContractDelegations::write(&delegations);

        ContractQueue::write_delegation_requests(DelegationKind::Delegate, request_queue);
        Ok(())
    }

    pub fn undelegate(
        &mut self,
        delegator: PublicKey,
        validator: PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()> {
        let mut request_queue = ContractQueue::read_delegation_requests::<UndelegateRequestKey>(
            DelegationKind::Undelegate,
        );

        let amount = match maybe_amount {
            None => U512::from(0),
            Some(amount) => amount,
        };

        request_queue.push(
            UndelegateRequestKey::new(delegator, validator),
            amount,
            self.get_block_time(),
        )?;

        ContractQueue::write_delegation_requests(DelegationKind::Undelegate, request_queue);

        // TODO: this should be factored out to ProofOfStake::step.
        self.step_undelegation(self.get_block_time())?;
        Ok(())
    }

    fn step_undelegation(&mut self, timestamp: BlockTime) -> Result<()> {
        let mut request_queue = ContractQueue::read_delegation_requests::<UndelegateRequestKey>(
            DelegationKind::Undelegate,
        );
        let requests = request_queue.pop_due(timestamp);

        let mut delegations = ContractDelegations::read()?;
        let mut stakes: Stakes = self.read()?;
        let pos_purse =
            get_purse(self, uref_names::POS_BONDING_PURSE).map_err(PurseLookupError::bonding)?;

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
            self.transfer_purse_to_account(pos_purse, delegator, payout)
                .map_err(|_| Error::UnbondTransferFailed)?;
        }

        ContractDelegations::write(&delegations);
        self.write(&stakes);
        ContractQueue::write_delegation_requests(DelegationKind::Undelegate, request_queue);
        Ok(())
    }

    pub fn redelegate(
        &mut self,
        delegator: PublicKey,
        src: PublicKey,
        dest: PublicKey,
        amount: U512,
    ) -> Result<()> {
        if src == dest {
            return Err(Error::SelfRedelegation);
        }

        let mut request_queue = ContractQueue::read_delegation_requests::<RedelegateRequestKey>(
            DelegationKind::Redelegate,
        );

        request_queue.push(
            RedelegateRequestKey::new(delegator, src, dest),
            amount,
            self.get_block_time(),
        )?;

        ContractQueue::write_delegation_requests(DelegationKind::Redelegate, request_queue);

        // TODO: this should be factored out to ProofOfStake::step.
        self.step_redelegation(self.get_block_time())?;

        Ok(())
    }

    fn step_redelegation(&mut self, timestamp: BlockTime) -> Result<()> {
        let mut request_queue = ContractQueue::read_delegation_requests::<RedelegateRequestKey>(
            DelegationKind::Redelegate,
        );

        let requests = request_queue.pop_due(timestamp);
        let mut delegations = ContractDelegations::read()?;
        let mut stakes: Stakes = self.read()?;

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
        self.write(&stakes);
        ContractQueue::write_delegation_requests(DelegationKind::Redelegate, request_queue);
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

    pub fn write_genesis_total_supply(&self, genesis_total_supply: &U512) -> Result<()> {
        let caller = runtime::get_caller();

        if caller.value() != consts::SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        let mut total_supply = ContractClaim::read_total_supply()?;
        total_supply.add(genesis_total_supply);
        ContractClaim::write_total_supply(&total_supply);

        Ok(())
    }

    pub fn distribute(&self) -> Result<()> {
        // 1. Increase total supply
        // 2. Do not mint in this phase.
        let mut total_supply = ContractClaim::read_total_supply()?;

        let delegations = ContractDelegations::read()?;
        let delegation_stat = ContractDelegations::read_stat()?;
        let delegation_sorted_stat = ContractDelegations::get_sorted_stat(&delegation_stat)?;

        let mut commissions = ContractClaim::read_commission()?;
        let mut rewards = ContractClaim::read_reward()?;

        // 1. Increase total supply
        //   U512::from(5) / U512::from(100) -> total inflation 5% per year
        //   U512::from(consts::DAYS_OF_YEAR * consts::HOURS_OF_DAY * consts::SECONDS_OF_HOUR
        //         * consts::BLOCK_PRODUCING_PER_SEC)
        //    -> divider for deriving inflation per block
        let inflation_pool_per_block = total_supply.0 * U512::from(5)
            / U512::from(
                100 * consts::DAYS_OF_YEAR
                    * consts::HOURS_OF_DAY
                    * consts::SECONDS_OF_HOUR
                    * consts::BLOCK_PRODUCING_PER_SEC,
            );
        total_supply.add(&inflation_pool_per_block);

        // Check total supply meets max supply
        if total_supply.0 > U512::from(consts::MAX_SUPPLY) * U512::from(consts::BIGSUN_TO_HDAC) {
            // No inflation anymore
            return Ok(());
        }

        ContractClaim::write_total_supply(&total_supply);

        /////////////////////////////////
        // Update validator's commission
        /////////////////////////////////
        //
        // 1. Check total delegations
        // 2. Pick 100 validators
        // 3. Summize it to derive total PoP.
        // 4. Calculate commission & add to commission claim table
        //
        // Check total delegations
        let mut total_delegation: U512 = U512::from(0);
        for (_, value) in delegation_stat.0.iter() {
            total_delegation += *value;
        }

        // Pick 100 validators + Summize it to derive total PoP
        let mut total_pop_score = U512::zero();
        let mut pop_score_table: BTreeMap<PublicKey, U512> = BTreeMap::new();
        for (idx, unit_data) in delegation_sorted_stat.into_iter().enumerate() {
            if idx >= 100 {
                break;
            }

            let unit_pop_score = pop_score_calculation(&total_delegation, &unit_data.amount);

            total_pop_score += unit_pop_score;
            pop_score_table.insert(unit_data.validator, unit_pop_score);
        }

        for (validator, unit_pop_score) in pop_score_table.iter() {
            let unit_commission = unit_pop_score
                * consts::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE
                * inflation_pool_per_block
                / (total_pop_score * U512::from(100));
            commissions.insert_commission(validator, &unit_commission);
        }
        ContractClaim::write_commission(&commissions);

        /////////////////////////////////
        // Update user's reward
        /////////////////////////////////
        // 1. Swipe delegation table, and derive user's portion of delegation
        // 2. Lookup delegation_stat table for total delegation for each validator
        // 3. Derive each validator's reward portion and insert reward of each user

        // 1. Swipe delegation table, and derive user's portion of delegation
        for (delegation_key, user_delegation_amount) in delegations.0.iter() {
            // 2. Lookup delegation_stat table for total delegation for each validator
            let total_delegation_per_validator = delegation_stat
                .0
                .get(&delegation_key.validator)
                .unwrap_or_revert_with(Error::DelegationsKeyDeserializationFailed);

            // 3. Derive each validator's reward portion and insert reward of each user
            let pop_score_of_validator = pop_score_table
                .get(&delegation_key.validator)
                .unwrap_or_revert_with(Error::DelegationsKeyDeserializationFailed);
            let user_reward = user_delegation_amount
                * pop_score_of_validator
                * U512::from(100 - consts::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE)
                * inflation_pool_per_block
                / (total_pop_score * U512::from(100) * total_delegation_per_validator);

            rewards.insert_rewards(&delegation_key.delegator, &user_reward);
        }
        ContractClaim::write_reward(&rewards);

        Ok(())
    }

    // For validator
    pub fn claim_commission(&self, validator: &PublicKey) -> Result<()> {
        // Processing commission claim table
        let mut commissions = ContractClaim::read_commission()?;
        let validator_commission = commissions
            .0
            .get(validator)
            .cloned()
            .unwrap_or_revert_with(Error::RewardNotFound);

        commissions.claim_commission(validator, &validator_commission);
        ContractClaim::write_commission(&commissions);

        let mut claim_requests = ContractQueue::read_claim_requests();

        claim_requests
            .0
            .push(ClaimRequest::Commission(*validator, validator_commission));

        ContractQueue::write_claim_requests(claim_requests);

        // Actual mint & transfer will be done at client-proxy
        Ok(())
    }

    // For user
    pub fn claim_reward(&self, user: &PublicKey) -> Result<()> {
        let mut rewards = ContractClaim::read_reward()?;
        let user_reward = rewards
            .0
            .get(user)
            .cloned()
            .unwrap_or_revert_with(Error::RewardNotFound);
        rewards.claim_rewards(user, &user_reward);
        ContractClaim::write_reward(&rewards);

        let mut claim_requests = ContractQueue::read_claim_requests();

        claim_requests
            .0
            .push(ClaimRequest::Reward(*user, user_reward));

        ContractQueue::write_claim_requests(claim_requests);

        // Actual mint & transfer will be done at client-proxy
        Ok(())
    }

    pub fn step_claim(&self) -> Result<()> {
        let claim_requests = ContractQueue::read_claim_requests();

        for request in claim_requests.0.iter() {
            let (pubkey, amount) = match request {
                ClaimRequest::Commission(pubkey, amount) | ClaimRequest::Reward(pubkey, amount) => {
                    (*pubkey, *amount)
                }
            };

            let mint_contract = system::get_mint();
            let minted_purse_res: core::result::Result<URef, mint::Error> =
                runtime::call_contract(mint_contract.clone(), ("mint", amount));
            let minted_purse = minted_purse_res.unwrap_or_revert();

            let transfer_res: TransferResult =
                system::transfer_from_purse_to_account(minted_purse, pubkey, amount);

            if let Err(err) = transfer_res {
                runtime::revert(err);
            }
        }

        // write an empty list.
        ContractQueue::write_claim_requests(Default::default());
        Ok(())
    }

    pub fn finalize_payment(&mut self, amount_spent: U512, _account: PublicKey) -> Result<()> {
        let caller = self.get_caller();
        if caller.value() != consts::SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        let payment_purse =
            get_purse(self, uref_names::POS_PAYMENT_PURSE).map_err(PurseLookupError::payment)?;
        let total = match self.balance(payment_purse) {
            Some(balance) => balance,
            None => return Err(Error::PaymentPurseBalanceNotFound),
        };
        if total < amount_spent {
            return Err(Error::InsufficientPaymentForAmountSpent);
        }

        // In the fare system, the fee is taken by the validator.
        let reward_purse =
            get_purse(self, uref_names::POS_REWARD_PURSE).map_err(PurseLookupError::rewards)?;

        self.transfer_purse_to_purse(payment_purse, reward_purse, total)
            .map_err(|_| Error::FailedTransferToRewardsPurse)?;

        Ok(())
    }
}

fn get_purse<R: RuntimeProvider>(
    runtime_provider: &R,
    name: &str,
) -> core::result::Result<URef, PurseLookupError> {
    runtime_provider
        .get_key(name)
        .ok_or(PurseLookupError::KeyNotFound)
        .and_then(|key| match key {
            Key::URef(uref) => Ok(uref),
            _ => Err(PurseLookupError::KeyUnexpectedType),
        })
}
