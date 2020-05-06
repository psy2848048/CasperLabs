use engine_shared::newtypes::Blake2bHash;
use types::ProtocolVersion;

use crate::engine_state::execution_effect::ExecutionEffect;

pub struct StepDelegationsRequest {
    pub parent_state_hash: Blake2bHash,
    pub block_time: u64,
    pub protocol_version: ProtocolVersion,
}

impl StepDelegationsRequest {
    pub fn new(
        parent_state_hash: Blake2bHash,
        block_time: u64,
        protocol_version: ProtocolVersion,
    ) -> Self {
        Self {
            parent_state_hash,
            block_time,
            protocol_version,
        }
    }
}

impl Default for StepDelegationsRequest {
    fn default() -> Self {
        Self {
            parent_state_hash: [0u8; 32].into(),
            block_time: 0,
            protocol_version: Default::default(),
        }
    }
}

pub enum StepDelegationsResult {
    RootNotFound(Blake2bHash),
    Success {
        post_state_hash: Blake2bHash,
        effect: ExecutionEffect,
    },
}
