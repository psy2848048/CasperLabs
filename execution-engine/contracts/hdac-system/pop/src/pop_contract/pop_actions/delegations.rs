use alloc::collections::BTreeMap;

use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    U512,
};

pub struct Delegations(pub BTreeMap<DelegationKey, U512>);
pub struct DelegationStat(pub BTreeMap<PublicKey, U512>);

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct DelegationKey {
    pub delegator: PublicKey,
    pub validator: PublicKey,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct DelegationUnitForOrder {
    pub validator: PublicKey,
    pub amount: U512,
}

impl Delegations {
    pub fn delegate(&mut self, delegator: &PublicKey, validator: &PublicKey, amount: U512) {
        let key = DelegationKey {
            delegator: *delegator,
            validator: *validator,
        };
        self.0
            .entry(key)
            .and_modify(|x| *x += amount)
            .or_insert(amount);
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

        match maybe_amount {
            // undelegate all
            None => self.0.remove(&key).ok_or(Error::NotDelegated),
            Some(amount) => {
                let delegation = self.0.get_mut(&key);
                match delegation {
                    Some(delegation) if *delegation > amount => {
                        *delegation -= amount;
                        Ok(amount)
                    }
                    Some(delegation) if *delegation == amount => {
                        self.0.remove(&key).ok_or(Error::DelegationsNotFound)
                    }
                    Some(_) => Err(Error::UndelegateTooLarge),
                    None => Err(Error::NotDelegated),
                }
            }
        }
    }
}
