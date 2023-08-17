use crate::prelude::*;

use crate::Error;

use ibc_proto::ibc::core::commitment::v1::MerklePath as RawMerklePath;
use ibc_types_domain_type::{DomainType, TypeUrl};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

impl TypeUrl for MerklePath {
    const TYPE_URL: &'static str = "/ibc.core.commitment.v1.MerklePath";
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
    fn try_from(value: RawMerklePath) -> Result<MerklePath, anyhow::Error> {
        Ok(MerklePath {
            key_path: value.key_path,
        })
    }
}
