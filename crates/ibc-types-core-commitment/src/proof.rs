use crate::prelude::*;

use crate::Error;

use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use ibc_types_domain_type::{DomainType, TypeUrl};
use ics23::CommitmentProof;

#[derive(Clone, Debug, PartialEq)]
pub struct MerkleProof {
    pub proofs: Vec<CommitmentProof>,
}

impl TypeUrl for MerkleProof {
    const TYPE_URL: &'static str = "/ibc.core.commitment.v1.MerkleProof";
}

impl DomainType for MerkleProof {
    type Proto = RawMerkleProof;
}

impl From<MerkleProof> for RawMerkleProof {
    fn from(value: MerkleProof) -> RawMerkleProof {
        RawMerkleProof {
            proofs: value.proofs,
        }
    }
}

impl TryFrom<RawMerkleProof> for MerkleProof {
    type Error = Error;
    fn try_from(value: RawMerkleProof) -> Result<MerkleProof, Error> {
        Ok(MerkleProof {
            proofs: value.proofs,
        })
    }
}
