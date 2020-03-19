use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::fmt::Write;

use contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

pub struct ContractVotes;
pub struct Votes(BTreeMap<VoteKey, U512>);
pub struct VoteStat(BTreeMap<PublicKey, U512>);

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct VoteKey {
    user: PublicKey,
    dapp: PublicKey,
}

impl ContractVotes {
    pub fn read() -> Result<Votes> {
        let mut votes = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("a") != split_name.next() {
                continue;
            }

            let to_publickey = |hex_str: &str| -> Result<PublicKey> {
                if hex_str.len() != 64 {
                    return Err(Error::VoteKeyDeserializationFailed);
                }
                let mut key_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut key_bytes)
                    .map_err(|_| Error::VoteKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == key_bytes.len());
                Ok(PublicKey::from(key_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::VoteKeyDeserializationFailed)?;
            let dapp_user = to_publickey(hex_key)?;

            let hex_key = split_name
                .next()
                .ok_or(Error::VoteKeyDeserializationFailed)?;
            let dapp_owner = to_publickey(hex_key)?;

            let balance = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::VotesDeserializationFailed)?;

            votes.insert(
                VoteKey {
                    user: dapp_user,
                    dapp: dapp_owner,
                },
                balance,
            );
        }
        if votes.is_empty() {
            return Err(Error::VotesNotFound);
        }
        Ok(Votes(votes))
    }

    /// Writes the current stakes to the contract's known urefs.
    pub fn write(votes: &Votes) {
        // Encode the stakes as a set of uref names.
        let mut new_urefs: BTreeSet<String> = votes
            .0
            .iter()
            .map(|(vote_key, balance)| {
                let to_hex_string = |address: PublicKey| -> String {
                    let bytes = address.value();
                    let mut ret = String::with_capacity(64);
                    for byte in &bytes[..32] {
                        write!(ret, "{:02x}", byte).expect("Writing to a string cannot fail");
                    }
                    ret
                };
                let user = to_hex_string(vote_key.user);
                let dapp = to_hex_string(vote_key.dapp);
                let mut uref = String::new();
                uref.write_fmt(format_args!("a_{}_{}_{}", user, dapp, balance))
                    .expect("Writing to a string cannot fail");
                uref
            })
            .collect();
        // Remove and add urefs to update the contract's known urefs accordingly.
        for (name, _) in runtime::list_named_keys() {
            if name.starts_with("a_") && !new_urefs.remove(&name) {
                runtime::remove_key(&name);
            }
        }
        for name in new_urefs {
            runtime::put_key(&name, Key::Hash([0; 32]));
        }
    }

    pub fn read_stat() -> Result<VoteStat> {
        let mut vote_stat = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("a") != split_name.next() {
                continue;
            }

            let to_publickey = |hex_str: &str| -> Result<PublicKey> {
                if hex_str.len() != 64 {
                    return Err(Error::VoteKeyDeserializationFailed);
                }
                let mut key_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut key_bytes)
                    .map_err(|_| Error::VoteKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == key_bytes.len());
                Ok(PublicKey::from(key_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::VoteKeyDeserializationFailed)?;
            let dapp_user = to_publickey(hex_key)?;

            let hex_key = split_name
                .next()
                .ok_or(Error::VoteKeyDeserializationFailed)?;
            let _dapp_owner = to_publickey(hex_key)?;

            let balance = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::VotesDeserializationFailed)?;

            let user_balance = vote_stat.entry(dapp_user).or_insert(U512::from(0));
            *user_balance += balance;
        }
        if vote_stat.is_empty() {
            return Err(Error::VotesNotFound);
        }

        Ok(VoteStat(vote_stat))
    }
}

impl Votes {
    pub fn vote(&mut self, user: &PublicKey, dapp: &PublicKey, amount: U512) {
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
        dapp: &PublicKey,
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

impl VoteStat {
    pub fn get(&mut self, user: &PublicKey) -> Result<U512>{
        Ok(*self.0.get(user).unwrap())
    }
}
