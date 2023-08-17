use crate::prelude::*;

use ibc_proto::ibc::core::commitment::v1::MerkleRoot as RawMerkleRoot;
use ibc_types_domain_type::{DomainType, TypeUrl};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleRoot {
    pub hash: Vec<u8>,
}

impl TypeUrl for MerkleRoot {
    const TYPE_URL: &'static str = "/ibc.core.commitment.v1.MerkleRoot";
}

impl DomainType for MerkleRoot {
    type Proto = RawMerkleRoot;
}

impl From<MerkleRoot> for RawMerkleRoot {
    fn from(value: MerkleRoot) -> RawMerkleRoot {
        RawMerkleRoot { hash: value.hash }
    }
}

impl TryFrom<RawMerkleRoot> for MerkleRoot {
    type Error = anyhow::Error;
    fn try_from(value: RawMerkleRoot) -> Result<MerkleRoot, Self::Error> {
        Ok(MerkleRoot { hash: value.hash })
    }
}
