use engine_grpc_server::engine_server::{ipc::StepRequest, state};
use types::{BlockTime, ProtocolVersion};

pub struct StepRequestBuilder {
    parent_state_hash: Vec<u8>,
    blocktime: u64,
    protocol_version: state::ProtocolVersion,
}

impl StepRequestBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_parent_state_hash(mut self, parent_state_hash: &[u8]) -> Self {
        self.parent_state_hash = parent_state_hash.to_vec();
        self
    }

    pub fn with_blocktime(mut self, blocktime: BlockTime) -> Self {
        self.blocktime = blocktime.into();
        self
    }

    pub fn with_protocol_version(mut self, protocol_version: ProtocolVersion) -> Self {
        self.protocol_version = protocol_version.into();
        self
    }

    pub fn build(self) -> StepRequest {
        let mut step_request = StepRequest::new();
        step_request.set_parent_state_hash(self.parent_state_hash);
        step_request.set_protocol_version(self.protocol_version);
        step_request.set_block_time(self.blocktime);
        step_request
    }
}

impl Default for StepRequestBuilder {
    fn default() -> Self {
        StepRequestBuilder {
            parent_state_hash: Default::default(),
            blocktime: Default::default(),
            protocol_version: ProtocolVersion::V1_0_0.into(),
        }
    }
}
