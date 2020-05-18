use std::convert::{TryFrom, TryInto};

use engine_core::engine_state::step::StepRequest;
use engine_shared::newtypes::BLAKE2B_DIGEST_LENGTH;

use crate::engine_server::ipc;

impl TryFrom<ipc::StepRequest> for StepRequest {
    type Error = ipc::StepResponse;

    fn try_from(mut request: ipc::StepRequest) -> Result<Self, Self::Error> {
        let parent_state_hash = {
            let parent_state_hash = request.take_parent_state_hash();
            let length = parent_state_hash.len();
            if length != BLAKE2B_DIGEST_LENGTH {
                let mut result = ipc::StepResponse::new();
                result.mut_missing_parent().set_hash(parent_state_hash);
                return Err(result);
            }
            parent_state_hash.as_slice().try_into().map_err(|_| {
                let mut result = ipc::StepResponse::new();
                result
                    .mut_missing_parent()
                    .set_hash(parent_state_hash.clone());
                result
            })?
        };

        let block_time = request.get_block_time();

        let protocol_version = request.take_protocol_version().into();

        Ok(StepRequest::new(
            parent_state_hash,
            block_time,
            protocol_version,
        ))
    }
}

impl From<StepRequest> for ipc::StepRequest {
    fn from(req: StepRequest) -> Self {
        let mut result = ipc::StepRequest::new();
        result.set_parent_state_hash(req.parent_state_hash.to_vec());
        result.set_block_time(req.block_time);
        result.set_protocol_version(req.protocol_version.into());
        result
    }
}
