use crate::math::sqrt_for_u512;
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

pub struct ContractClaim;

// TODO: How to inject the # of total supply in genesis block?
// Seems to needed one more step in CLI to gather that
//   as like collect-gentxs
pub struct TotalSupply(pub U512);
pub struct Commissions(pub BTreeMap<PublicKey, U512>);
pub struct Rewards(pub BTreeMap<PublicKey, U512>);

pub const INFLATION_COMMISSION: &str = "ic";
pub const FARE_COMMISSION: &str = "fc";
pub const INFLATION_REWARD: &str = "ir";
pub const FARE_REWARD: &str = "fr";

pub fn pop_score_calculation(total_delegated: &U512, validator_delegated_amount: &U512) -> U512 {
    // Currenrly running in PoS.
    // Profession factor will be added soon
    let profession_factor = U512::from(1);

    let x = *validator_delegated_amount * U512::from(100) / *total_delegated;

    let score = if x <= U512::from(15) {
        // y = 1000x
        *validator_delegated_amount * U512::from(100_000) / *total_delegated
    } else {
        // y = 1000 * sqrt(30x - 225)
        //   = sqrt(1_000_000 * 30 * 100 * val_delegation / total_delegated - 225_000_000)
        sqrt_for_u512(
            *validator_delegated_amount * U512::from(3_000_000_000_u64) / *total_delegated
                - U512::from(225_000_000),
        )
    };

    score * profession_factor
}

impl ContractClaim {
    // prefix: "t"
    // t_{total_supply}
    pub fn read_total_supply() -> Result<TotalSupply> {
        let mut total_supply = U512::from(0);
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("t") != split_name.next() {
                continue;
            }

            total_supply = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::TotalSupplyDeserializationFailed)?;

            break;
        }

        Ok(TotalSupply(total_supply))
    }

    // prefix: "t"
    // t_{total_supply}
    pub fn write_total_supply(total_supply: &TotalSupply) {
        for (name, _) in runtime::list_named_keys() {
            if name.starts_with("t_") {
                runtime::remove_key(&name);
                break;
            }
        }
        let mut uref = String::new();
        uref.write_fmt(format_args!("t_{}", total_supply.0))
            .expect("Writing to a string cannot fail");
        runtime::put_key(&uref, Key::Hash([0; 32]));
    }

    // prefix: "cps"
    // cps_{commission_purse_snapshot}
    pub fn read_commission_purse_snapshot() -> Result<U512> {
        let mut amount = U512::from(0);

        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("cps") != split_name.next() {
                continue;
            }

            amount = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::CommissionPurseSnapshotDeserializationFailed)?;

            break;
        }

        Ok(amount)
    }

    // prefix: "cps"
    // cps_{commission_purse_snapshot}
    pub fn write_commission_purse_snapshot(amount: U512) {
        for (name, _) in runtime::list_named_keys() {
            if name.starts_with("cps_") {
                runtime::remove_key(&name);
                break;
            }
        }
        let mut uref = String::new();
        uref.write_fmt(format_args!("cps_{}", amount))
            .expect("Writing to a string cannot fail");
        runtime::put_key(&uref, Key::Hash([0; 32]));
    }

    // prefix: "rps"
    // rps_{reward_purse_snapshot}
    pub fn read_reward_purse_snapshot() -> Result<U512> {
        let mut amount = U512::from(0);

        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("rps") != split_name.next() {
                continue;
            }

            amount = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::RewardPurseSnapshotDeserializationFailed)?;

            break;
        }

        Ok(amount)
    }

    // prefix: "rps"
    // rps_{reward_purse_snapshot}
    pub fn write_reward_purse_snapshot(amount: U512) {
        for (name, _) in runtime::list_named_keys() {
            if name.starts_with("rps_") {
                runtime::remove_key(&name);
                break;
            }
        }
        let mut uref = String::new();
        uref.write_fmt(format_args!("rps_{}", amount))
            .expect("Writing to a string cannot fail");
        runtime::put_key(&uref, Key::Hash([0; 32]));
    }

    // prefix: "c", "cp"
    // c_{PublicKey}_{ClaimableBalance}
    pub fn read_commission(prefix: &str) -> Result<Commissions> {
        let mut commissions = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some(prefix) != split_name.next() {
                continue;
            }

            let to_publickey = |hex_str: &str| -> Result<PublicKey> {
                if hex_str.len() != 64 {
                    return Err(Error::CommissionKeyDeserializationFailed);
                }
                let mut key_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut key_bytes)
                    .map_err(|_| Error::CommissionKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == key_bytes.len());
                Ok(PublicKey::ed25519_from(key_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::CommissionKeyDeserializationFailed)?;
            let validator = to_publickey(hex_key)?;

            let balance = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::CommissionBalanceDeserializationFailed)?;

            commissions.insert(validator, balance);
        }

        Ok(Commissions(commissions))
    }

    // prefix: "c", "cp"
    // c_{PublicKey}_{ClaimableBalance}
    pub fn write_commission(prefix: &str, commissions: &Commissions) {
        // Encode the stakes as a set of uref names.
        let mut new_urefs: BTreeSet<String> = commissions
            .0
            .iter()
            .map(|(pubkey, balance)| {
                let to_hex_string = |address: PublicKey| -> String {
                    let bytes = address.value();
                    let mut ret = String::with_capacity(64);
                    for byte in &bytes[..32] {
                        write!(ret, "{:02x}", byte).expect("Writing to a string cannot fail");
                    }
                    ret
                };

                let validator = to_hex_string(*pubkey);
                let mut uref = String::new();
                uref.write_fmt(format_args!("{}_{}_{}", prefix, validator, balance))
                    .expect("Writing to a string cannot fail");
                uref
            })
            .collect();

        // Remove and add urefs to update the contract's known urefs accordingly.
        for (name, _) in runtime::list_named_keys() {
            let mut prefix_: String = String::new();
            prefix_
                .write_fmt(format_args!("{}_", prefix))
                .expect("Writing to a string cannot fail");
            if name.starts_with(&prefix_) && !new_urefs.remove(&name) {
                runtime::remove_key(&name);
            }
        }
        for name in new_urefs {
            runtime::put_key(&name, Key::Hash([0; 32]));
        }
    }

    // prefix: "r", "rp"
    // r_{PublicKey}_{ClaimableBalance}
    pub fn read_reward(prefix: &str) -> Result<Rewards> {
        let mut rewards = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some(prefix) != split_name.next() {
                continue;
            }

            let to_publickey = |hex_str: &str| -> Result<PublicKey> {
                if hex_str.len() != 64 {
                    return Err(Error::RewardKeyDeserializationFailed);
                }
                let mut key_bytes = [0u8; 32];
                let _bytes_written = base16::decode_slice(hex_str, &mut key_bytes)
                    .map_err(|_| Error::RewardKeyDeserializationFailed)?;
                debug_assert!(_bytes_written == key_bytes.len());
                Ok(PublicKey::ed25519_from(key_bytes))
            };

            let hex_key = split_name
                .next()
                .ok_or(Error::RewardKeyDeserializationFailed)?;
            let user = to_publickey(hex_key)?;

            let balance = split_name
                .next()
                .and_then(|b| U512::from_dec_str(b).ok())
                .ok_or(Error::RewardBalanceDeserializationFailed)?;

            rewards.insert(user, balance);
        }

        Ok(Rewards(rewards))
    }

    // prefix: "r", "rp"
    // r_{PublicKey}_{ClaimableBalance}
    pub fn write_reward(prefix: &str, rewards: &Rewards) {
        // Encode the stakes as a set of uref names.
        let mut new_urefs: BTreeSet<String> = rewards
            .0
            .iter()
            .map(|(pubkey, balance)| {
                let to_hex_string = |address: PublicKey| -> String {
                    let bytes = address.value();
                    let mut ret = String::with_capacity(64);
                    for byte in &bytes[..32] {
                        write!(ret, "{:02x}", byte).expect("Writing to a string cannot fail");
                    }
                    ret
                };

                let user = to_hex_string(*pubkey);
                let mut uref = String::new();
                uref.write_fmt(format_args!("{}_{}_{}", prefix, user, balance))
                    .expect("Writing to a string cannot fail");
                uref
            })
            .collect();

        // Remove and add urefs to update the contract's known urefs accordingly.
        for (name, _) in runtime::list_named_keys() {
            let mut prefix_: String = String::new();
            prefix_
                .write_fmt(format_args!("{}_", prefix))
                .expect("Writing to a string cannot fail");
            if name.starts_with(&prefix_) && !new_urefs.remove(&name) {
                runtime::remove_key(&name);
            }
        }
        for name in new_urefs {
            runtime::put_key(&name, Key::Hash([0; 32]));
        }
    }
}

impl TotalSupply {
    pub fn add(&mut self, amount: &U512) {
        self.0 += *amount;
    }
}

impl Commissions {
    pub fn insert_commission(&mut self, validator: &PublicKey, amount: &U512) {
        self.0
            .entry(*validator)
            .and_modify(|x| *x += *amount)
            .or_insert(*amount);
    }

    pub fn claim_commission(&mut self, validator: &PublicKey, amount: &U512) {
        let claim = self.0.get_mut(validator);
        match claim {
            Some(claim) if *claim > *amount => {
                *claim -= *amount;
            }
            Some(claim) if *claim == *amount => {
                self.0
                    .remove(validator)
                    .ok_or(Error::CommissionClaimRecordNotFound)
                    .unwrap_or_default();
            }
            Some(_) => runtime::revert(Error::CommissionClaimTooLarge),
            None => runtime::revert(Error::NoCommission),
        }
    }
}

impl Rewards {
    pub fn insert_rewards(&mut self, user: &PublicKey, amount: &U512) {
        self.0
            .entry(*user)
            .and_modify(|x| *x += *amount)
            .or_insert(*amount);
    }

    pub fn claim_rewards(&mut self, user: &PublicKey, amount: &U512) {
        let claim = self.0.get_mut(user);
        match claim {
            Some(claim) if *claim > *amount => {
                *claim -= *amount;
            }
            Some(claim) if *claim == *amount => {
                self.0
                    .remove(user)
                    .ok_or(Error::RewardClaimRecordNotFound)
                    .unwrap_or_default();
            }
            Some(_) => runtime::revert(Error::RewardClaimTooLarge),
            None => runtime::revert(Error::NoReward),
        }
    }
}
