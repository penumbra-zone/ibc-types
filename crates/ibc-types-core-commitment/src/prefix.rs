use crate::prelude::*;
use crate::MerklePath;

use crate::Error;

use ibc_proto::ibc::core::commitment::v1::MerklePrefix as RawMerklePrefix;
use ibc_types_domain_type::{DomainType, TypeUrl};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerklePrefix {
    pub key_prefix: Vec<u8>,
}

impl MerklePrefix {
    /// apply the prefix to the supplied paths
    pub fn apply(&self, paths: Vec<String>) -> MerklePath {
        let commitment_str =
            core::str::from_utf8(&self.key_prefix).expect("commitment prefix is not valid utf-8");
        let mut key_path: Vec<String> = vec![format!("{commitment_str:?}")];
        key_path.append(paths.clone().as_mut());

        MerklePath { key_path }
    }
}

impl TypeUrl for MerklePrefix {
    const TYPE_URL: &'static str = "/ibc.core.commitment.v1.MerklePrefix";
}

impl DomainType for MerklePrefix {
    type Proto = RawMerklePrefix;
}

impl From<Vec<u8>> for MerklePrefix {
    fn from(value: Vec<u8>) -> MerklePrefix {
        MerklePrefix { key_prefix: value }
    }
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
