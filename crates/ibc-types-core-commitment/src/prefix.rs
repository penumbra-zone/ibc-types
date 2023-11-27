use crate::prelude::*;
use crate::MerklePath;

use ibc_proto::ibc::core::commitment::v1::MerklePrefix as RawMerklePrefix;
use ibc_types_domain_type::DomainType;

#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
#[cfg_attr(
    feature = "with_serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "RawMerklePrefix", into = "RawMerklePrefix")
)]
pub struct MerklePrefix {
    pub key_prefix: Vec<u8>,
}

impl MerklePrefix {
    /// apply the prefix to the supplied paths
    pub fn apply(&self, paths: Vec<String>) -> MerklePath {
        let commitment_str =
            core::str::from_utf8(&self.key_prefix).expect("commitment prefix is not valid utf-8");
        let mut key_path: Vec<String> = vec![commitment_str.to_string()];
        key_path.append(paths.clone().as_mut());

        MerklePath { key_path }
    }
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

impl From<RawMerklePrefix> for MerklePrefix {
    fn from(value: RawMerklePrefix) -> MerklePrefix {
        MerklePrefix {
            key_prefix: value.key_prefix,
        }
    }
}
