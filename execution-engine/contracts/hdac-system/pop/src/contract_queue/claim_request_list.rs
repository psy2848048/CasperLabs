use alloc::{boxed::Box, vec::Vec};
use core::result;

use types::{
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped,
};

use super::requests::ClaimRequest;

#[derive(Default)]
pub struct ClaimRequestList(pub Vec<ClaimRequest>);

impl FromBytes for ClaimRequestList {
    fn from_bytes(bytes: &[u8]) -> result::Result<(Self, &[u8]), bytesrepr::Error> {
        let (size, rest): (u32, &[u8]) = FromBytes::from_bytes(bytes)?;

        let mut result = Vec::new();
        let mut stream = rest;
        for _ in 0..size {
            let (t, rem): (ClaimRequest, &[u8]) = FromBytes::from_bytes(stream)?;
            result.push(t);
            stream = rem;
        }
        Ok((ClaimRequestList(result), stream))
    }
}

impl ToBytes for ClaimRequestList {
    fn to_bytes(&self) -> result::Result<Vec<u8>, bytesrepr::Error> {
        let size = self.0.len() as u32;
        let mut result: Vec<u8> = Vec::new();
        result.extend(size.to_bytes()?);
        result.extend(
            self.0
                .iter()
                .map(ToBytes::to_bytes)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten(),
        );
        Ok(result)
    }
}

impl CLTyped for ClaimRequestList {
    fn cl_type() -> CLType {
        CLType::List(Box::new(ClaimRequest::cl_type()))
    }
}
