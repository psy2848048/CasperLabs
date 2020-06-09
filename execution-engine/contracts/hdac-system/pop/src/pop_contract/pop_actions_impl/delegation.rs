use alloc::{
    collections::{btree_map::Iter, BTreeMap},
    vec::Vec,
};

use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    U512,
};

use crate::store;

const MAX_VALIDATORS: usize = 100;

pub struct Delegations {
    table: BTreeMap<DelegationKey, U512>,
    total_amount: Option<U512>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct DelegationKey {
    pub delegator: PublicKey,
    pub validator: PublicKey,
}

impl Delegations {
    pub fn new(table: BTreeMap<DelegationKey, U512>) -> Self {
        Self {
            table,
            total_amount: None,
        }
    }

    pub fn iter(&self) -> Iter<DelegationKey, U512> {
        self.table.iter()
    }

    pub fn validators(&self) -> Vec<(PublicKey, U512)> {
        let mut validators = BTreeMap::default();
        for (
            DelegationKey {
                delegator,
                validator,
            },
            amount,
        ) in self.table.iter()
        {
            validators
                .entry(*validator)
                .and_modify(|x| *x += *amount)
                .or_insert(*amount);
        }

        let mut validators = validators.into_iter().collect::<Vec<_>>();

        // sort by descending order and truncate
        validators.sort_by(|a, b| b.1.cmp(&a.1));
        validators.truncate(MAX_VALIDATORS);

        validators
    }

    pub fn total_amount(&self) -> U512 {
        self.total_amount
            .unwrap_or_else(|| self.table.values().fold(U512::zero(), |acc, x| acc + x))
    }

    pub fn delegation(&self, delegator: PublicKey, validator: PublicKey) -> Result<U512> {
        self.table
            .get(&DelegationKey {
                delegator,
                validator,
            })
            .cloned()
            .ok_or(Error::DelegationsNotFound)
    }

    pub fn delegating_amount(&self, delegator: &PublicKey) -> U512 {
        self.table
            .iter()
            .map(|x| {
                if x.0.delegator == *delegator {
                    *x.1
                } else {
                    U512::zero()
                }
            })
            .fold(U512::zero(), |acc, x| acc + x)
    }

    pub fn delegated_amount(&self, validator: &PublicKey) -> U512 {
        self.table
            .iter()
            .filter(|x| x.0.validator == *validator)
            .map(|x| x.1)
            .fold(U512::zero(), |acc, x| acc + *x)
    }

    pub fn delegate(
        &mut self,
        delegator: &PublicKey,
        validator: &PublicKey,
        amount: U512,
    ) -> Result<()> {
        let key = DelegationKey {
            delegator: *delegator,
            validator: *validator,
        };

        // validate amount
        {
            let bonding_amount = store::read_bonding_amount(delegator);
            let delegating_amount = self.delegating_amount(delegator);
            if amount > bonding_amount.saturating_sub(delegating_amount) {
                // TODO: return Err(Error::TryToDelegateMoreThanStakes);
                return Err(Error::UndelegateTooLarge);
            }
        }

        // update table
        self.table
            .entry(key)
            .and_modify(|x| *x += amount)
            .or_insert(amount);

        // update total amount
        self.total_amount = match self.total_amount {
            Some(total_amount) => Some(total_amount + amount),
            None => Some(amount),
        };

        Ok(())
    }

    pub fn undelegate(
        &mut self,
        delegator: &PublicKey,
        validator: &PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<U512> {
        let key = DelegationKey {
            delegator: *delegator,
            validator: *validator,
        };

        // update table
        let undelegate_amount = match maybe_amount {
            // undelegate all
            None => self.table.remove(&key).ok_or(Error::NotDelegated)?,
            Some(amount) => {
                let delegation = self.table.get_mut(&key);
                match delegation {
                    Some(delegation) if *delegation > amount => {
                        *delegation -= amount;
                        amount
                    }
                    Some(delegation) if *delegation == amount => {
                        self.table.remove(&key).ok_or(Error::DelegationsNotFound)?
                    }
                    Some(_) => return Err(Error::UndelegateTooLarge),
                    None => return Err(Error::NotDelegated),
                }
            }
        };

        // update total amount
        self.total_amount = match self.total_amount {
            Some(total_amount) => Some(total_amount.saturating_sub(undelegate_amount)),
            None => unreachable!(),
        };

        Ok(undelegate_amount)
    }

    pub fn redelegate(
        &mut self,
        delegator: &PublicKey,
        src_validator: &PublicKey,
        dest_validator: &PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()> {
        // update table
        {
            // undelegate
            let key = DelegationKey {
                delegator: *delegator,
                validator: *src_validator,
            };
            let undelegate_amount = match maybe_amount {
                // undelegate all
                None => self.table.remove(&key).ok_or(Error::NotDelegated)?,
                Some(amount) => {
                    let delegation = self.table.get_mut(&key);
                    match delegation {
                        Some(delegation) if *delegation > amount => {
                            *delegation -= amount;
                            amount
                        }
                        Some(delegation) if *delegation == amount => {
                            self.table.remove(&key).ok_or(Error::DelegationsNotFound)?
                        }
                        Some(_) => return Err(Error::UndelegateTooLarge),
                        None => return Err(Error::NotDelegated),
                    }
                }
            };

            // delegate
            let key = DelegationKey {
                delegator: *delegator,
                validator: *dest_validator,
            };
            self.table
                .entry(key)
                .and_modify(|x| *x += undelegate_amount)
                .or_insert(undelegate_amount);
        }

        Ok(())
    }
}
