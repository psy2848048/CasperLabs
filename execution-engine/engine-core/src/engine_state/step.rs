use std::fmt;

use engine_shared::{newtypes::Blake2bHash, TypeMismatch};
use engine_storage::global_state::CommitResult;
use types::{bytesrepr, Key, ProtocolVersion};

use crate::engine_state::execution_effect::ExecutionEffect;

pub struct StepRequest {
    pub parent_state_hash: Blake2bHash,
    pub block_time: u64,
    pub block_height: u64,
    pub protocol_version: ProtocolVersion,
}

impl StepRequest {
    pub fn new(
        parent_state_hash: Blake2bHash,
        block_time: u64,
        block_height: u64,
        protocol_version: ProtocolVersion,
    ) -> Self {
        Self {
            parent_state_hash,
            block_time,
            block_height,
            protocol_version,
        }
    }
}

impl Default for StepRequest {
    fn default() -> Self {
        Self {
            parent_state_hash: [0u8; 32].into(),
            block_time: 0,
            block_height: 0,
            protocol_version: Default::default(),
        }
    }
}

pub enum StepResult {
    RootNotFound(Blake2bHash),
    KeyNotFound(Key),
    TypeMismatch(TypeMismatch),
    Serialization(bytesrepr::Error),
    Success {
        post_state_hash: Blake2bHash,
        effect: ExecutionEffect,
    },
}

impl fmt::Display for StepResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::RootNotFound(hash) => write!(f, "Root not found: {}", hash),
            Self::KeyNotFound(key) => write!(f, "Key not found: {}", key),
            Self::TypeMismatch(type_mismatch) => write!(f, "Type mismatch: {:?}", type_mismatch),
            Self::Serialization(error) => write!(f, "Serialization error: {:?}", error),
            Self::Success {
                post_state_hash,
                effect,
            } => write!(f, "Success: {} {:?}", post_state_hash, effect),
        }
    }
}

impl StepResult {
    pub fn from_commit_result(
        commit_result: CommitResult,
        parent_state_hash: Blake2bHash,
        effect: ExecutionEffect,
    ) -> Self {
        match commit_result {
            CommitResult::RootNotFound => Self::RootNotFound(parent_state_hash),
            CommitResult::KeyNotFound(key) => Self::KeyNotFound(key),
            CommitResult::TypeMismatch(type_mismatch) => Self::TypeMismatch(type_mismatch),
            CommitResult::Serialization(error) => Self::Serialization(error),
            CommitResult::Success { state_root, .. } => Self::Success {
                post_state_hash: state_root,
                effect,
            },
        }
    }
}
