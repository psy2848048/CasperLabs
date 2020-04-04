#![allow(clippy::clone_on_copy)]

use alloc::collections::BTreeMap;
use contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use proof_of_stake::{MintProvider, ProofOfStake, RuntimeProvider, Stakes, StakesProvider};
use types::{
    account::{PublicKey, PurseId},
    system_contract_errors::pos::{Error, PurseLookupError, Result},
    BlockTime, Key, URef, U512,
};

use crate::{
    constants::{consts, local_keys, uref_names},
    contract_delegations::ContractDelegations,
    contract_economy::{pop_score_calculation, ContractClaim},
    contract_mint::ContractMint,
    contract_queue::{
        ClaimKeyType, ClaimRequestKey, ContractQueue, DelegateRequestKey, RedelegateRequestKey,
        UndelegateRequestKey,
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
    pub fn step(&self) -> Result<()> {
        // let blocktime = runtime::get_blocktime();
        // self.step_undelegation(blocktime);
        // self.step_redelegation(blocktime);
        let _ = self.distribute();
        let _ = self.step_claim();

        Ok(())
    }

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

    pub fn write_genesis_total_supply(&self, genesis_total_supply: &U512) -> Result<()> {
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
        //   U512::from(consts::DAYS_OF_YEAR * consts::HOURS_OF_DAY * consts::SECONDS_OF_HOUR /
        //         consts::BLOCK_TIME_IN_SEC)
        //    -> divider for deriving inflation per block
        let inflation_pool_per_block = total_supply.0 * U512::from(5)
            / U512::from(
                100 * consts::DAYS_OF_YEAR * consts::HOURS_OF_DAY * consts::SECONDS_OF_HOUR
                    / consts::BLOCK_TIME_IN_SEC,
            );
        total_supply.add(&inflation_pool_per_block);
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
            .unwrap_or_revert_with(Error::RewardNotFound);
        let validator_commission_clone = validator_commission.clone();
        commissions.claim_commission(validator, &validator_commission_clone);
        ContractClaim::write_commission(&commissions);

        // Pushing into queue
        let mut claim_commission_queue =
            ContractQueue::read_claim_requests::<ClaimRequestKey>(local_keys::CLAIM_REQUEST_QUEUE);

        claim_commission_queue.push(
            ClaimRequestKey::new(ClaimKeyType::Commission, *validator),
            validator_commission_clone,
        )?;

        ContractQueue::write_claim_requests(
            local_keys::CLAIM_REQUEST_QUEUE,
            claim_commission_queue,
        );

        // Actual mint & transfer will be done at client-proxy
        Ok(())
    }

    // For user
    pub fn claim_reward(&self, user: &PublicKey) -> Result<()> {
        let mut rewards = ContractClaim::read_reward()?;
        let user_reward = rewards
            .0
            .get(user)
            .unwrap_or_revert_with(Error::RewardNotFound);
        let user_reward_clone = user_reward.clone();
        rewards.claim_rewards(user, &user_reward_clone);
        ContractClaim::write_reward(&rewards);

        // Pushing into queue
        let mut claim_reward_queue =
            ContractQueue::read_claim_requests::<ClaimRequestKey>(local_keys::CLAIM_REQUEST_QUEUE);

        claim_reward_queue.push(
            ClaimRequestKey::new(ClaimKeyType::Reward, *user),
            user_reward_clone,
        )?;

        ContractQueue::write_claim_requests(local_keys::CLAIM_REQUEST_QUEUE, claim_reward_queue);

        // Actual mint & transfer will be done at client-proxy
        Ok(())
    }

    pub fn step_claim(&self) -> Result<()> {
        let mut claim_queue =
            ContractQueue::read_claim_requests::<ClaimRequestKey>(local_keys::CLAIM_REQUEST_QUEUE);

        let queue_clone_for_iter = claim_queue.0.clone();
        for unit_claim_entry in queue_clone_for_iter.iter() {
            let mint_contract_uref = system::get_mint();

            // "mint" cannot be called from outside. "create" is alternative
            let temp_purse =
                runtime::call_contract(mint_contract_uref, ("create", unit_claim_entry.amount));
            let _ = system::transfer_from_purse_to_account(
                temp_purse,
                unit_claim_entry.request_key.pubkey,
                unit_claim_entry.amount,
            );

            claim_queue.pop(unit_claim_entry.request_key);
        }

        ContractQueue::write_claim_requests(local_keys::CLAIM_REQUEST_QUEUE, claim_queue);

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
