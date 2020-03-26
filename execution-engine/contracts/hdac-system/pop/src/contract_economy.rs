use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
};
use core::fmt::Write;

use contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use types::{
    account::PublicKey,
    bytesrepr::ToBytes,
    system_contract_errors::pos::{Error, Result},
    ApiError, Key, U512,
};

pub struct ContractClaim;

// TODO: How to inject the # of total supply in genesis block?
// Seems to needed one more step in CLI to gather that
//   as like collect-gentxs
pub struct TotalSupply(pub U512);
pub struct Commissions(pub BTreeMap<PublicKey, U512>);
pub struct Rewards(pub BTreeMap<PublicKey, U512>);

// Used for dapp gas deduction rate calculation
const CONST: f64 = 87_000_000_f64;

pub fn sum_of_delegation(commissions: &Commissions) -> Result<U512> {
    let res = U512::from(0);
    // U512 has no implementation of 'sum' in for-each.
    for (_, value) in commissions.0.iter() {
        res += *value;
    }
    Ok(res)
}

pub fn pop_score_calculation(total_delegated: &U512, validator_delegated_amount: &U512) -> f64 {
    // Currenrly running in PoS.
    // Profession factor will be added soon
    let mut score: f64 = 0_f64;
    let x: f64 = u512ToF64(*validator_delegated_amount) / u512ToF64(*total_delegated) * 100.0_f64;
    let profession_factor = 1_f64;

    if x <= 15.0_f64 {
        score = x;
    } else {
        score = 22.2561_f64 * (x + 7.2561_f64).ln() - 54.0521_f64;
    }

    score * profession_factor
}

pub fn dapp_gas_deduction_rate_calculation(dapp_voted_amount: U512) -> f64 {
    let dapp_voted_amount_converted = u512ToF64(dapp_voted_amount);
    dapp_voted_amount_converted * (1_f64 + CONST / (2_f64 * dapp_voted_amount_converted)).ln() / CONST
}

pub fn u512ToF64(uint512_number: U512) -> f64 {
    let mut uint512_str = String::new();
    uint512_str.write_fmt(format_args!("{}", uint512_number)).expect("Writing to a string cannot fail");
    uint512_str.parse().unwrap()
}

pub fn f64ToU512(f64_number: f64) -> U512 {
    let mut f64_str = String::new();
    f64_str.write_fmt(format_args!("{}", f64_str)).expect("Writing to a string cannot fail");
    let mut split_number = f64_str.split('.');
    let decimal_str = split_number.next().unwrap_or_revert_with(Error::UintParsingError);

    U512::from_dec_str(decimal_str).ok().unwrap_or_revert_with(Error::UintParsingError)
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

            let res = split_name
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

                let mut uref = String::new();
                uref.write_fmt(format_args!("t_{}", total_supply.0))
                    .expect("Writing to a string cannot fail");
                runtime::put_key(&uref, Key::Hash([0; 32]));

                return;
            }
        }
    }

    // prefix: "c"
    // c_{PublicKey}_{ClaimableBalance}
    #[allow(clippy::or_fun_call)]
    pub fn read_commission() -> Result<Commissions> {
        let mut commissions = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("c") != split_name.next() {
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
                Ok(PublicKey::from(key_bytes))
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

    // prefix: "c"
    // c_{PublicKey}_{ClaimableBalance}
    #[allow(clippy::or_fun_call)]
    pub fn write_commission(commissions: &Commissions) {
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
                uref.write_fmt(format_args!("c_{}_{}", validator, balance))
                    .expect("Writing to a string cannot fail");
                uref
            })
            .collect();

        // Remove and add urefs to update the contract's known urefs accordingly.
        for (name, _) in runtime::list_named_keys() {
            if name.starts_with("c_") && !new_urefs.remove(&name) {
                runtime::remove_key(&name);
            }
        }
        for name in new_urefs {
            runtime::put_key(&name, Key::Hash([0; 32]));
        }
    }

    // prefix: "r"
    // r_{PublicKey}_{ClaimableBalance}
    #[allow(clippy::or_fun_call)]
    pub fn read_reward() -> Result<Rewards> {
        let mut rewards = BTreeMap::new();
        for (name, _) in runtime::list_named_keys() {
            let mut split_name = name.split('_');
            if Some("r") != split_name.next() {
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
                Ok(PublicKey::from(key_bytes))
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

    // prefix: "r"
    // r_{PublicKey}_{ClaimableBalance}
    #[allow(clippy::or_fun_call)]
    pub fn write_reward(rewards: &Rewards) {
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
                uref.write_fmt(format_args!("r_{}_{}", user, balance))
                    .expect("Writing to a string cannot fail");
                uref
            })
            .collect();

        // Remove and add urefs to update the contract's known urefs accordingly.
        for (name, _) in runtime::list_named_keys() {
            if name.starts_with("r_") && !new_urefs.remove(&name) {
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
                self.0.remove(validator).ok_or(Error::CommissionClaimRecordNotFound);
            }
            None => runtume::revert(Error::NoCommission),
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
                self.0.remove(user).ok_or(Error::RewardClaimRecordNotFound);
            }
            None => runtime::revert(Error::NoReward),
        }
    }
}
