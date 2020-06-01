pub mod delegations;
pub mod delegations_provider;
#[allow(dead_code)] // this mod comes from CasperLabs' stakes
pub mod stakes;
pub mod stakes_provider;
pub mod votes;
pub mod votes_provider;

use types::{account::PublicKey, system_contract_errors::pos::Result, Key, URef, U512};

pub trait ProofOfProfession: Delegatable + Votable + Stakable {}

pub trait Stakable {
    fn bond(&mut self, caller: PublicKey, amount: U512, source_purse: URef) -> Result<()>;
    fn unbond(&mut self, caller: PublicKey, maybe_amount: Option<U512>) -> Result<()>;
}

pub trait Delegatable {
    fn delegate(&mut self, delegator: PublicKey, validator: PublicKey, amount: U512) -> Result<()>;

    fn undelegate(
        &mut self,
        delegator: PublicKey,
        validator: PublicKey,
        maybe_amount: Option<U512>,
    ) -> Result<()>;

    fn redelegate(
        &mut self,
        delegator: PublicKey,
        src: PublicKey,
        dest: PublicKey,
        amount: Option<U512>,
    ) -> Result<()>;
}

pub trait Votable {
    fn vote(&mut self, user: PublicKey, dapp: Key, amount: U512) -> Result<()>;
    fn unvote(&mut self, user: PublicKey, dapp: Key, maybe_amount: Option<U512>) -> Result<()>;
}
