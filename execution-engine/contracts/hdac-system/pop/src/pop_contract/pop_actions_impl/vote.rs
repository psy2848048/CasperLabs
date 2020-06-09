use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

use crate::store;

pub fn vote(voter: &PublicKey, dapp: &Key, amount: U512) -> Result<()> {
    // validate amount
    if amount.is_zero() {
        // TODO: change to Error::VoteTooSmall
        return Err(Error::BondTooSmall);
    }

    let bonding_amount = store::read_bonding_amount(voter);
    let voting_amount = store::read_voting_amount(voter);

    if amount > bonding_amount.saturating_sub(voting_amount) {
        return Err(Error::VoteTooLarge);
    }

    // update voting amount (voter, amount)
    store::write_voting_amount(voter, voting_amount + amount);

    // update vote ((voter, dapp), amount)
    let current_amount = store::read_vote(voter, dapp);
    store::write_vote(voter, dapp, current_amount + amount);

    // update voted amount (dapp, amount)
    let current_amount = store::read_voted_amount(dapp);
    store::write_voted_amount(dapp, current_amount + amount);

    Ok(())
}

pub fn unvote(voter: &PublicKey, dapp: &Key, amount: U512) -> Result<()> {
    // update vote ((voter, dapp), amount)
    let vote_amount = store::read_vote(voter, dapp);
    if amount > vote_amount {
        return Err(Error::UnvoteTooLarge);
    }
    store::write_vote(voter, dapp, vote_amount - amount);

    // update voting amount (voter, amount)
    let current_amount: U512 = store::read_voting_amount(voter);
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    store::write_voting_amount(voter, current_amount.saturating_sub(amount));

    // update voted amount (dapp, amount)
    let current_amount: U512 = store::read_voted_amount(dapp);
    // if amount > current_amount {
    //     Err(Error::InternalError);
    // }
    store::write_voted_amount(dapp, current_amount.saturating_sub(amount));

    Ok(())
}
