use crate::prelude::*;

use crate::Error;

use ibc_proto::ibc::core::commitment::v1::MerklePrefix as RawMerklePrefix;
use ibc_types_domain_type::{DomainType, TypeUrl};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerklePrefix {
    pub key_prefix: Vec<u8>,
}

impl TypeUrl for MerklePrefix {
    const TYPE_URL: &'static str = "/ibc.core.commitment.v1.MerklePrefix";
}

impl DomainType for MerklePrefix {
    type Proto = RawMerklePrefix;
}

impl From<MerklePrefix> for RawMerklePrefix {
    fn from(value: MerklePrefix) -> RawMerklePrefix {
        RawMerklePrefix {
            key_prefix: value.key_prefix,
        }
    }
}

impl TryFrom<RawMerklePrefix> for MerklePrefix {
    type Error = Error;
    fn try_from(value: RawMerklePrefix) -> Result<MerklePrefix, Error> {
        Ok(MerklePrefix {
            key_prefix: value.key_prefix,
        })
    }
}
