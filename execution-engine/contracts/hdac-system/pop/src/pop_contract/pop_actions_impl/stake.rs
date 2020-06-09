use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    U512,
};

use crate::store;

pub fn bond(user: &PublicKey, amount: U512) {
    let bonding_amount = store::read_bonding_amount(*user);
    store::write_bonding_amount(*user, bonding_amount + amount);
}

pub fn unbond(user: &PublicKey, maybe_amount: Option<U512>) -> Result<U512> {
    let bonding_amount = store::read_bonding_amount(*user);

    let unbond_amount = match maybe_amount {
        Some(amount) => amount,
        None => bonding_amount,
    };

    // validate amount
    {
        let max_action_amount = U512::max(
            // TODO: expensive operation, reflect a better way
            store::read_delegations()?.delegating_amount(user),
            store::read_voting_amount(*user),
        );
        if unbond_amount > bonding_amount.saturating_sub(max_action_amount) {
            return Err(Error::UnbondTooLarge);
        }
    }

    store::write_bonding_amount(*user, bonding_amount.saturating_sub(unbond_amount));
    Ok(unbond_amount)
}
