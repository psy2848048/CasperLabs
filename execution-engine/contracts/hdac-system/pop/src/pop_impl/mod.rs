mod economy;
mod pop_actions;
mod request_pool;

pub use pop_actions::{Delegatable, Votable};

use alloc::collections::BTreeMap;
use contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};

use types::{
    account::PublicKey,
    system_contract_errors::{
        mint,
        pos::{Error, PurseLookupError, Result},
    },
    AccessRights, BlockTime, Key, TransferResult, URef, U512,
};

use crate::constants::{sys_params, uref_names};

use economy::{pop_score_calculation, ContractClaim, INFLATION_COMMISSION, INFLATION_REWARD};
use pop_actions::ProofOfProfession;
use request_pool::{
    ClaimRequest, ContractQueue, DelegationKind, RedelegateRequestKey, UndelegateRequestKey,
};

const SYSTEM_ACCOUNT: [u8; 32] = [0u8; 32];
const DAYS_OF_YEAR: i64 = 365_i64;
const HOURS_OF_DAY: i64 = 24_i64;
const SECONDS_OF_HOUR: i64 = 3600_i64;

pub struct ProofOfProfessionContract;

impl ProofOfProfession for ProofOfProfessionContract {}

impl Delegatable for ProofOfProfessionContract {
    fn delegate(
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
            get_purse(uref_names::POS_BONDING_PURSE).map_err(PurseLookupError::bonding)?;

        system::transfer_from_purse_to_purse(source_purse, pos_purse, amount)
            .map_err(|_| Error::BondTransferFailed)?;

        // check validator is bonded
        let mut stakes = self.read_stakes()?;
        // if this is not self-delegation and target validator is not bonded
        if delegator != validator && !stakes.0.contains_key(&validator) {
            return Err(Error::NotBonded);
        }

        let mut delegations = self.read_delegations()?;

        stakes.bond(&validator, amount);
        delegations.delegate(&delegator, &validator, amount);

        self.write_stakes(&stakes);
        self.write_delegations(&delegations);

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
            runtime::get_blocktime(),
        )?;

        ContractQueue::write_delegation_requests(DelegationKind::Undelegate, request_queue);
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

        let mut request_queue = ContractQueue::read_delegation_requests::<RedelegateRequestKey>(
            DelegationKind::Redelegate,
        );

        request_queue.push(
            RedelegateRequestKey::new(delegator, src, dest),
            amount,
            runtime::get_blocktime(),
        )?;

        ContractQueue::write_delegation_requests(DelegationKind::Redelegate, request_queue);
        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        let caller = runtime::get_caller();

        if caller.value() != SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        let current = runtime::get_blocktime();
        self.step_undelegation(
            current.saturating_sub(BlockTime::new(sys_params::UNBONDING_DELAY)),
        )?;
        self.step_redelegation(
            current.saturating_sub(BlockTime::new(sys_params::UNBONDING_DELAY)),
        )?;

        // TODO: separate to another function
        self.distribute()?;
        self.step_claim()?;

        Ok(())
    }
}

impl Votable for ProofOfProfessionContract {
    fn vote(&self, user: PublicKey, dapp: Key, amount: U512) -> Result<()> {
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

    fn unvote(&self, user: PublicKey, dapp: Key, maybe_amount: Option<U512>) -> Result<()> {
        let mut votes = self.read_votes()?;
        votes.unvote(&user, &dapp, maybe_amount)?;
        self.write_votes(&votes);

        Ok(())
    }
}

impl ProofOfProfessionContract {
    // For validator
    pub fn claim_commission(&self, validator: &PublicKey) -> Result<()> {
        // Processing commission claim table
        let mut commissions = ContractClaim::read_commission(INFLATION_COMMISSION)?;
        let validator_commission = commissions
            .0
            .get(validator)
            .cloned()
            .unwrap_or_revert_with(Error::RewardNotFound);

        commissions.claim_commission(validator, &validator_commission);
        ContractClaim::write_commission(INFLATION_COMMISSION, &commissions);

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
        let mut rewards = ContractClaim::read_reward(INFLATION_REWARD)?;
        let user_reward = rewards
            .0
            .get(user)
            .cloned()
            .unwrap_or_revert_with(Error::RewardNotFound);
        rewards.claim_rewards(user, &user_reward);
        ContractClaim::write_reward(INFLATION_REWARD, &rewards);

        let mut claim_requests = ContractQueue::read_claim_requests();

        claim_requests
            .0
            .push(ClaimRequest::Reward(*user, user_reward));

        ContractQueue::write_claim_requests(claim_requests);

        // Actual mint & transfer will be done at client-proxy
        Ok(())
    }

    pub fn get_payment_purse(&self) -> Result<URef> {
        let purse = get_purse(uref_names::POS_PAYMENT_PURSE).map_err(PurseLookupError::payment)?;
        // Limit the access rights so only balance query and deposit are allowed.
        Ok(URef::new(purse.addr(), AccessRights::READ_ADD))
    }

    pub fn finalize_payment(&mut self, amount_spent: U512, _account: PublicKey) -> Result<()> {
        let caller = runtime::get_caller();
        if caller.value() != SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        let payment_purse =
            get_purse(uref_names::POS_PAYMENT_PURSE).map_err(PurseLookupError::payment)?;
        let total = match system::get_balance(payment_purse) {
            Some(balance) => balance,
            None => return Err(Error::PaymentPurseBalanceNotFound),
        };
        if total < amount_spent {
            return Err(Error::InsufficientPaymentForAmountSpent);
        }

        // In the fare system, the fee is taken by the validator.
        let reward_purse =
            get_purse(uref_names::POS_REWARD_PURSE).map_err(PurseLookupError::rewards)?;
        let commission_purse =
            get_purse(uref_names::POS_COMMISSION_PURSE).map_err(PurseLookupError::commission)?;

        let reward_amount =
            total * sys_params::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE / U512::from(100);
        let commission_amount = total
            * (U512::from(100) - sys_params::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE)
            / U512::from(100);

        if total != (reward_amount + commission_amount) {
            let remain_amount = total - reward_amount - commission_amount;

            let communtiy_purse =
                get_purse(uref_names::POS_COMMUNITY_PURSE).map_err(PurseLookupError::communtiy)?;

            system::transfer_from_purse_to_purse(payment_purse, communtiy_purse, remain_amount)
                .map_err(|_| Error::FailedTransferToCommunityPurse)?;
        }

        system::transfer_from_purse_to_purse(payment_purse, reward_purse, reward_amount)
            .map_err(|_| Error::FailedTransferToRewardsPurse)?;

        system::transfer_from_purse_to_purse(payment_purse, commission_purse, commission_amount)
            .map_err(|_| Error::FailedTransferToCommissionPurse)?;

        Ok(())
    }

    fn distribute(&self) -> Result<()> {
        // 1. Increase total supply
        // 2. Do not mint in this phase.
        let mut total_supply = ContractClaim::read_total_supply()?;

        let delegations = self.read_delegations()?;
        let delegation_stat = self.read_delegation_stat()?;
        let delegation_sorted_stat = self.get_sorted_delegation_stat(&delegation_stat)?;

        let mut commissions = ContractClaim::read_commission(INFLATION_COMMISSION)?;
        let mut rewards = ContractClaim::read_reward(INFLATION_REWARD)?;

        // 1. Increase total supply
        //   U512::from(5) / U512::from(100) -> total inflation 5% per year
        //   U512::from(DAYS_OF_YEAR * HOURS_OF_DAY * SECONDS_OF_HOUR
        //         * sys_params::BLOCK_PRODUCING_PER_SEC)
        //    -> divider for deriving inflation per block
        let inflation_pool_per_block = total_supply.0 * U512::from(5)
            / U512::from(
                100 * DAYS_OF_YEAR
                    * HOURS_OF_DAY
                    * SECONDS_OF_HOUR
                    * sys_params::BLOCK_PRODUCING_PER_SEC,
            );
        total_supply.add(&inflation_pool_per_block);

        // Check total supply meets max supply
        if total_supply.0
            > U512::from(sys_params::MAX_SUPPLY) * U512::from(sys_params::BIGSUN_TO_HDAC)
        {
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
                * sys_params::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE
                * inflation_pool_per_block
                / (total_pop_score * U512::from(100));
            commissions.insert_commission(validator, &unit_commission);
        }
        ContractClaim::write_commission(INFLATION_COMMISSION, &commissions);

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
                * U512::from(100 - sys_params::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE)
                * inflation_pool_per_block
                / (total_pop_score * U512::from(100) * total_delegation_per_validator);

            rewards.insert_rewards(&delegation_key.delegator, &user_reward);
        }
        ContractClaim::write_reward(INFLATION_REWARD, &rewards);

        Ok(())
    }

    fn step_undelegation(&mut self, due: BlockTime) -> Result<()> {
        let mut request_queue = ContractQueue::read_delegation_requests::<UndelegateRequestKey>(
            DelegationKind::Undelegate,
        );
        let requests = request_queue.pop_due(due);

        let mut delegations = self.read_delegations()?;
        let mut stakes = self.read_stakes()?;
        let pos_purse =
            get_purse(uref_names::POS_BONDING_PURSE).map_err(PurseLookupError::bonding)?;

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
            system::transfer_from_purse_to_account(pos_purse, delegator, payout)
                .map_err(|_| Error::UnbondTransferFailed)?;
        }

        self.write_delegations(&delegations);
        self.write_stakes(&stakes);
        ContractQueue::write_delegation_requests(DelegationKind::Undelegate, request_queue);
        Ok(())
    }

    fn step_redelegation(&mut self, due: BlockTime) -> Result<()> {
        let mut request_queue = ContractQueue::read_delegation_requests::<RedelegateRequestKey>(
            DelegationKind::Redelegate,
        );

        let requests = request_queue.pop_due(due);
        let mut delegations = self.read_delegations()?;
        let mut stakes = self.read_stakes()?;

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

        self.write_delegations(&delegations);
        self.write_stakes(&stakes);
        ContractQueue::write_delegation_requests(DelegationKind::Redelegate, request_queue);
        Ok(())
    }

    fn step_claim(&self) -> Result<()> {
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
}

fn get_purse(name: &str) -> core::result::Result<URef, PurseLookupError> {
    runtime::get_key(name)
        .ok_or(PurseLookupError::KeyNotFound)
        .and_then(|key| match key {
            Key::URef(uref) => Ok(uref),
            _ => Err(PurseLookupError::KeyUnexpectedType),
        })
}
