pub mod delegations;
pub mod delegations_provider;
#[allow(dead_code)] // this mod comes from CasperLabs' stakes
pub mod stakes;
pub mod stakes_provider;
pub mod votes;
pub mod votes_provider;

use types::{account::PublicKey, system_contract_errors::pos::Result, Key, URef, U512};

pub trait ProofOfProfession: Delegatable + Votable {}

pub trait Delegatable {
    fn delegate(
        &mut self,
        delegator: PublicKey,
        validator: PublicKey,
        amount: U512,
        source_purse: URef,
    ) -> Result<()>;

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
        amount: U512,
    ) -> Result<()>;

    // execute the mature (un,re)delegation requests
    fn step(&mut self) -> Result<()>;
}

pub trait Votable {
    fn vote(&self, user: PublicKey, dapp: Key, amount: U512) -> Result<()>;
    fn unvote(&self, user: PublicKey, dapp: Key, maybe_amount: Option<U512>) -> Result<()>;
}
