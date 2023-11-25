use crate::prelude::*;

use ibc_proto::ibc::core::commitment::v1::MerklePath as RawMerklePath;
use ibc_types_domain_type::DomainType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

impl DomainType for MerklePath {
    type Proto = RawMerklePath;
}

impl From<MerklePath> for RawMerklePath {
    fn from(value: MerklePath) -> RawMerklePath {
        RawMerklePath {
            key_path: value.key_path,
        }
    }
}

impl TryFrom<RawMerklePath> for MerklePath {
    type Error = anyhow::Error;
    fn try_from(value: RawMerklePath) -> Result<MerklePath, Self::Error> {
        Ok(MerklePath {
            key_path: value.key_path,
        })
    }
}
