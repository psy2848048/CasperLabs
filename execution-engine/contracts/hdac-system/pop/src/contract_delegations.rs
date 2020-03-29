use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};
use core::fmt::Write;

use contract::contract_api::runtime;
use types::{
    account::PublicKey,
    system_contract_errors::pos::{Error, Result},
    Key, U512,
};

pub struct ContractDelegations;
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

impl ContractDelegations {
    pub fn read() -> Result<Delegations> {
        let mut delegations = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("d") != split_name.next() {
                continue;
            }

            let to_publickey = |hex_str: &str| -> Result<PublicKey> {
                if hex_str.len() != 64 {
                    return Err(Error::DelegationsKeyDeserializationFailed);
                }
                let mut key_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut key_bytes)
                    .map_err(|_| Error::DelegationsKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == key_bytes.len());
                Ok(PublicKey::from(key_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::DelegationsKeyDeserializationFailed)?;
            let delegator = to_publickey(hex_key)?;

            let hex_key = split_name
                .next()
                .ok_or(Error::DelegationsKeyDeserializationFailed)?;
            let validator = to_publickey(hex_key)?;

            let balance = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::DelegationsDeserializationFailed)?;

            delegations.insert(
                DelegationKey {
                    delegator,
                    validator,
                },
                balance,
            );
        }
        if delegations.is_empty() {
            return Err(Error::DelegationsNotFound);
        }
        Ok(Delegations(delegations))
    }

    /// Writes the current stakes to the contract's known urefs.
    pub fn write(delegations: &Delegations) {
        // Encode the stakes as a set of uref names.
        let mut new_urefs: BTreeSet<String> = delegations
            .0
            .iter()
            .map(|(delegation_key, balance)| {
                let to_hex_string = |address: PublicKey| -> String {
                    let bytes = address.value();
                    let mut ret = String::with_capacity(64);
                    for byte in &bytes[..32] {
                        write!(ret, "{:02x}", byte).expect("Writing to a string cannot fail");
                    }
                    ret
                };
                let delegator = to_hex_string(delegation_key.delegator);
                let validator = to_hex_string(delegation_key.validator);
                let mut uref = String::new();
                uref.write_fmt(format_args!("d_{}_{}_{}", delegator, validator, balance))
                    .expect("Writing to a string cannot fail");
                uref
            })
            .collect();
        // Remove and add urefs to update the contract's known urefs accordingly.
        for (name, _) in runtime::list_named_keys() {
            if name.starts_with("d_") && !new_urefs.remove(&name) {
                runtime::remove_key(&name);
            }
        }
        for name in new_urefs {
            runtime::put_key(&name, Key::Hash([0; 32]));
        }
    }

    pub fn read_stat() -> Result<DelegationStat> {
        let mut delegation_stat = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("d") != split_name.next() {
                continue;
            }

            let to_publickey = |hex_str: &str| -> Result<PublicKey> {
                if hex_str.len() != 64 {
                    return Err(Error::DelegationsKeyDeserializationFailed);
                }
                let mut key_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut key_bytes)
                    .map_err(|_| Error::DelegationsKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == key_bytes.len());
                Ok(PublicKey::from(key_bytes))
            };

            let hex_key = split_name
                .nth(1)
                .ok_or(Error::DelegationsKeyDeserializationFailed)?;
            let validator = to_publickey(hex_key)?;

            let balance = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::DelegationsDeserializationFailed)?;

            let delegation_balance = delegation_stat
                .entry(validator)
                .or_insert_with(|| U512::from(0));
            *delegation_balance += balance;
        }

        Ok(DelegationStat(delegation_stat))
    }

    pub fn read_user_stat() -> Result<DelegationStat> {
        let mut delegation_stat = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("d") != split_name.next() {
                continue;
            }

            let to_publickey = |hex_str: &str| -> Result<PublicKey> {
                if hex_str.len() != 64 {
                    return Err(Error::DelegationsKeyDeserializationFailed);
                }
                let mut key_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut key_bytes)
                    .map_err(|_| Error::DelegationsKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == key_bytes.len());
                Ok(PublicKey::from(key_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::DelegationsKeyDeserializationFailed)?;
            let delegator = to_publickey(hex_key)?;

            let balance = split_name
                .nth(1)
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::DelegationsDeserializationFailed)?;

            let delegation_balance = delegation_stat
                .entry(delegator)
                .or_insert_with(|| U512::from(0));
            *delegation_balance += balance;
        }

        Ok(DelegationStat(delegation_stat))
    }

    pub fn get_sorted_stat(
        delegation_stat: &DelegationStat,
    ) -> Result<Vec<DelegationUnitForOrder>> {
        let mut delegation_sorted: Vec<DelegationUnitForOrder> = Vec::new();
        for (key, value) in delegation_stat.0.iter() {
            let unit = DelegationUnitForOrder {
                validator: *key,
                amount: *value,
            };
            delegation_sorted.push(unit);
        }
        delegation_sorted.sort_by(|a, b| b.amount.cmp(&a.amount));

        Ok(delegation_sorted)
    }
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
