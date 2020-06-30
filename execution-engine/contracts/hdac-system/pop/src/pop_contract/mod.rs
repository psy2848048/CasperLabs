mod economy;
mod pop_actions;
mod pop_actions_impl;

pub use pop_actions::{Delegatable, Stakable, Votable};
pub use pop_actions_impl::{DelegationKey, Delegations};

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

use crate::{
    constants::{sys_params, uref_names},
    store::{self, ClaimRequest, RedelegateRequest, UnbondRequest, UndelegateRequest},
};

use economy::pop_score_calculation;
use pop_actions_impl::stake;

const DAYS_OF_YEAR: i64 = 365_i64;
const HOURS_OF_DAY: i64 = 24_i64;
const SECONDS_OF_HOUR: i64 = 3600_i64;

pub struct ProofOfProfessionContract;

impl ProofOfProfessionContract {
    pub fn install_genesis_states(
        &mut self,
        total_mint_supply: U512,
        genesis_validators: BTreeMap<PublicKey, U512>,
    ) -> Result<()> {
        if runtime::get_caller().value() != sys_params::SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        // write the total mint supply state
        store::write_total_mint_supply(total_mint_supply);

        // write stake and delegation states
        let mut delegations = store::read_delegations()?;

        for (validator, amount) in &genesis_validators {
            // bond and write self-delegation
            stake::bond(validator, *amount);
            delegations.delegate(&validator, &validator, *amount)?;
        }

        store::write_delegations(&delegations);
        Ok(())
    }

    pub fn step(&mut self) -> Result<()> {
        let caller = runtime::get_caller();

        if caller.value() != sys_params::SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        // The order of below functions matters.
        let current = runtime::get_blocktime();
        let mut delegations = store::read_delegations()?;

        // step mature undelegate requests
        {
            // populate the requests.
            let mut request_queue = store::read_undelegation_requests();
            let requests = request_queue.pop_due(
                current.saturating_sub(BlockTime::new(sys_params::UNDELEGATING_DELAY_IN_SEC)),
            );
            store::write_undelegation_requests(request_queue);

            for request in requests {
                let UndelegateRequest {
                    delegator,
                    validator,
                    maybe_amount,
                } = request.item;
                // If the request is invalid, discard the request.
                // TODO: Error is ignored currently, but should propagate to endpoint in the future.
                let _ = delegations.undelegate(&delegator, &validator, maybe_amount);
            }
        }

        // step the mature redelegate requests
        {
            // populate the requests.
            let mut request_queue = store::read_redelegation_requests();
            let requests = request_queue.pop_due(
                current.saturating_sub(BlockTime::new(sys_params::UNDELEGATING_DELAY_IN_SEC)),
            );
            store::write_redelegation_requests(request_queue);

            for request in requests {
                let RedelegateRequest {
                    delegator,
                    src_validator,
                    dest_validator,
                    maybe_amount,
                } = request.item;

                // If the request is invalid, discard the request.
                // TODO: Error is currently ignored, but should propagate to endpoint in the future.
                let _ = delegations.redelegate(
                    &delegator,
                    &src_validator,
                    &dest_validator,
                    maybe_amount,
                );
            }
        }

        store::write_delegations(&delegations);

        self.step_unbond(
            current.saturating_sub(BlockTime::new(sys_params::UNBONDING_DELAY_IN_SEC)),
            &delegations,
        );

        // TODO: separate to another function
        let _ = self.distribute(&delegations);
        let _ = self.step_claim();

        Ok(())
    }

    // For validator
    pub fn claim_commission(&mut self, validator: &PublicKey) -> Result<()> {
        // Processing commission claim table
        let commission_amount = store::read_commission_amount(validator);
        store::write_commission_amount(validator, U512::zero());

        let mut claim_requests = store::read_claim_requests();
        claim_requests.push(ClaimRequest::Commission(*validator, commission_amount));
        store::write_claim_requests(claim_requests);

        // Actual mint & transfer will be done at client-proxy
        Ok(())
    }

    // For user
    pub fn claim_reward(&mut self, user: &PublicKey) -> Result<()> {
        let reward_amount = store::read_reward_amount(user);
        store::write_reward_amount(user, U512::zero());

        let mut claim_requests = store::read_claim_requests();
        claim_requests.push(ClaimRequest::Reward(*user, reward_amount));
        store::write_claim_requests(claim_requests);

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
        if caller.value() != sys_params::SYSTEM_ACCOUNT {
            return Err(Error::SystemFunctionCalledByUserAccount);
        }

        let payment_purse =
            get_purse(uref_names::POS_PAYMENT_PURSE).map_err(PurseLookupError::payment)?;
        let reward_amount = match system::get_balance(payment_purse) {
            Some(balance) => balance,
            None => return Err(Error::PaymentPurseBalanceNotFound),
        };
        if reward_amount < amount_spent {
            return Err(Error::InsufficientPaymentForAmountSpent);
        }

        // In the fare system, the fee is taken by the validator.
        let reward_purse =
            get_purse(uref_names::POS_REWARD_PURSE).map_err(PurseLookupError::rewards)?;

        system::transfer_from_purse_to_purse(payment_purse, reward_purse, reward_amount)
            .map_err(|_| Error::FailedTransferToRewardsPurse)?;

        Ok(())
    }

    fn distribute(&mut self, delegations: &Delegations) -> Result<()> {
        // 1. Increase total supply
        // 2. Do not mint in this phase.
        let mut total_supply = store::read_total_mint_supply();

        // 1. Increase total supply
        //   U512::from(5) / U512::from(100) -> total inflation 5% per year
        //   U512::from(DAYS_OF_YEAR * HOURS_OF_DAY * SECONDS_OF_HOUR
        //         * sys_params::BLOCK_PRODUCING_PER_SEC)
        //    -> divider for deriving inflation per block
        let inflation_pool_per_block = total_supply * U512::from(5)
            / U512::from(
                100 * DAYS_OF_YEAR
                    * HOURS_OF_DAY
                    * SECONDS_OF_HOUR
                    * sys_params::BLOCK_PRODUCING_PER_SEC,
            );
        total_supply += inflation_pool_per_block;

        // Check total supply meets max supply
        if total_supply
            > U512::from(sys_params::MAX_SUPPLY) * U512::from(sys_params::BIGSUN_TO_HDAC)
        {
            // No inflation anymore
            return Ok(());
        }

        store::write_total_mint_supply(total_supply);

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
        let total_delegation = delegations.total_amount();

        // Pick 100 validators + Summize it to derive total PoP
        let mut total_pop_score = U512::zero();
        let mut pop_score_table: BTreeMap<PublicKey, U512> = BTreeMap::new();
        let validators = delegations.validators();
        for (validator, delegated_amount) in &validators {
            let unit_pop_score = pop_score_calculation(&total_delegation, &delegated_amount);

            total_pop_score += unit_pop_score;
            pop_score_table.insert(*validator, unit_pop_score);
        }

        for (validator, unit_pop_score) in pop_score_table.iter() {
            let unit_commission = unit_pop_score
                * sys_params::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE
                * inflation_pool_per_block
                / (total_pop_score * U512::from(100));

            let current = store::read_commission_amount(validator);
            store::write_commission_amount(validator, current + unit_commission);
        }

        /////////////////////////////////
        // Update user's reward
        /////////////////////////////////
        // 1. Swipe delegation table, and derive user's portion of delegation
        // 2. Lookup delegation_stat table for total delegation for each validator
        // 3. Derive each validator's reward portion and insert reward of each user

        // 1. Swipe delegation table, and derive user's portion of delegation
        for (
            DelegationKey {
                delegator,
                validator,
            },
            user_delegation_amount,
        ) in delegations.iter()
        {
            // 2. Lookup delegation_stat table for total delegation for each validator
            let total_delegation_per_validator = delegations.delegated_amount(validator);

            // 3. Derive each validator's reward portion and insert reward of each user
            let pop_score_of_validator = pop_score_table
                .get(validator)
                .ok_or(Error::DelegationsKeyDeserializationFailed)?;

            let user_reward = user_delegation_amount
                * pop_score_of_validator
                * U512::from(100 - sys_params::VALIDATOR_COMMISSION_RATE_IN_PERCENTAGE)
                * inflation_pool_per_block
                / (total_pop_score * U512::from(100) * total_delegation_per_validator);

            let current = store::read_reward_amount(delegator);
            store::write_reward_amount(validator, current + user_reward);
        }

        Ok(())
    }

    fn step_unbond(&mut self, due: BlockTime, delegations: &Delegations) {
        // populate the mature requests
        let mut request_queue = store::read_unbond_requests();
        let requests = request_queue.pop_due(due);
        store::write_unbond_requests(request_queue);

        for request in requests {
            let UnbondRequest {
                requester,
                maybe_amount,
            } = request.item;

            // If the request is invalid, discard the request.
            // TODO: Error is ignored currently, but should propagate to endpoint in the future.
            if let Ok(payout) = stake::unbond(&requester, maybe_amount, delegations) {
                if let Ok(pos_purse) = get_purse(uref_names::POS_BONDING_PURSE) {
                    let _ = system::transfer_from_purse_to_account(pos_purse, requester, payout);
                }
            }
        }
    }

    fn step_claim(&mut self) -> Result<()> {
        let claim_requests = store::read_claim_requests();

        for request in claim_requests.iter() {
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
        store::write_claim_requests(Default::default());
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
