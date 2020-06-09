mod delegations;
mod delegations_provider;

pub use delegations::{DelegationKey, Delegations};
pub use delegations_provider::{read_delegations, write_delegations};
