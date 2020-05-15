use alloc::collections::BTreeMap;

use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

pub struct Votes(pub BTreeMap<VoteKey, U512>);
pub struct VoteStat(pub BTreeMap<PublicKey, U512>);

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct VoteKey {
    pub user: PublicKey,
    pub dapp: Key,
}

impl Votes {
    pub fn vote(&mut self, user: &PublicKey, dapp: &Key, amount: U512) {
        let key = VoteKey {
            user: *user,
            dapp: *dapp,
        };
        self.0
            .entry(key)
            .and_modify(|x| *x += amount)
            .or_insert(amount);
    }

    pub fn unvote(
        &mut self,
        user: &PublicKey,
        dapp: &Key,
        maybe_amount: Option<U512>,
    ) -> Result<U512> {
        let key = VoteKey {
            user: *user,
            dapp: *dapp,
        };

        match maybe_amount {
            // undelegate all
            None => self.0.remove(&key).ok_or(Error::NotVoted),
            Some(amount) => {
                let vote = self.0.get_mut(&key);
                match vote {
                    Some(vote) if *vote > amount => {
                        *vote -= amount;
                        Ok(amount)
                    }
                    Some(vote) if *vote == amount => {
                        self.0.remove(&key).ok_or(Error::VotesNotFound)
                    }
                    Some(_) => Err(Error::UnvoteTooLarge),
                    None => Err(Error::NotVoted),
                }
            }
        }
    }
}
