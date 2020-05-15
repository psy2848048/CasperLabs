use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::fmt::Write;

use contract::contract_api::runtime;
use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

use super::votes::{VoteKey, VoteStat, Votes};
use crate::pop_impl::ProofOfProfessionContract;

impl ProofOfProfessionContract {
    pub fn read_votes(&self) -> Result<Votes> {
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
                Ok(PublicKey::ed25519_from(key_bytes))
            };

            let to_hash = |hex_str: &str| -> Result<Key> {
                if hex_str.len() != 64 {
                    return Err(Error::VoteKeyDeserializationFailed);
                }
                let mut hash_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut hash_bytes)
                    .map_err(|_| Error::VoteKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == hash_bytes.len());
                // TODO: How to convert hash to key
                Ok(Key::Hash(hash_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::VoteKeyDeserializationFailed)?;
            let dapp_user = to_publickey(hex_key)?;

            let hex_key = split_name
                .next()
                .ok_or(Error::VoteKeyDeserializationFailed)?;
            let dapp_owner = to_hash(hex_key)?;

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

        Ok(Votes(votes))
    }

    /// Writes the current stakes to the contract's known urefs.
    pub fn write_votes(&self, votes: &Votes) {
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

                let to_hex_string_from_hash = |hash: Key| -> String {
                    let bytes = hash.into_hash().expect("VoteKey serialization cannot fail");
                    let mut ret = String::with_capacity(64);
                    for byte in &bytes[..32] {
                        write!(ret, "{:02x}", byte).expect("Writing to a string cannot fail");
                    }
                    ret
                };

                let user = to_hex_string(vote_key.user);
                let dapp = to_hex_string_from_hash(vote_key.dapp);
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

    pub fn read_vote_stat(&self) -> Result<VoteStat> {
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
                Ok(PublicKey::ed25519_from(key_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::VoteKeyDeserializationFailed)?;
            let dapp_user = to_publickey(hex_key)?;

            let balance = split_name
                .nth(1)
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::VotesDeserializationFailed)?;

            let user_balance = vote_stat.entry(dapp_user).or_insert_with(|| U512::from(0));
            *user_balance += balance;
        }

        Ok(VoteStat(vote_stat))
    }
}
