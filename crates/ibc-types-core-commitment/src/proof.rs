use crate::prelude::*;

use crate::MerklePath;
use crate::MerkleRoot;

use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use ibc_types_domain_type::{DomainType, TypeUrl};
use ics23::commitment_proof::Proof;
use ics23::CommitmentProof;
use ics23::{
    calculate_existence_root, verify_membership, verify_non_membership, NonExistenceProof,
};

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

/// Convert to ics23::CommitmentProof
/// The encoding and decoding shouldn't fail since ics23::CommitmentProof and ibc_proto::ics23::CommitmentProof should be the same
/// Ref. <https://github.com/informalsystems/ibc-rs/issues/853>
impl TryFrom<RawMerkleProof> for MerkleProof {
    type Error = anyhow::Error;
    fn try_from(proof: RawMerkleProof) -> Result<Self, Self::Error> {
        let proofs: Vec<CommitmentProof> = proof
            .proofs
            .into_iter()
            .map(|p| {
                let mut encoded = Vec::new();
                prost::Message::encode(&p, &mut encoded).unwrap();
                prost::Message::decode(&*encoded).unwrap()
            })
            .collect();
        Ok(Self { proofs })
    }
}

// TODO move to ics23
fn calculate_non_existence_root(proof: &NonExistenceProof) -> Result<Vec<u8>, anyhow::Error> {
    if let Some(left) = &proof.left {
        calculate_existence_root::<ics23::HostFunctionsManager>(left)
            .map_err(|_| anyhow::anyhow!("invalid merkle proof"))
    } else if let Some(right) = &proof.right {
        calculate_existence_root::<ics23::HostFunctionsManager>(right)
            .map_err(|_| anyhow::anyhow!("invalid merkle proof"))
    } else {
        Err(anyhow::anyhow!("invalid merkle proof"))
    }
}

impl MerkleProof {
    pub fn verify_membership(
        &self,
        specs: &[ics23::ProofSpec],
        root: MerkleRoot,
        keys: MerklePath,
        value: Vec<u8>,
        start_index: usize,
    ) -> Result<(), anyhow::Error> {
        // validate arguments
        if self.proofs.is_empty() {
            return Err(anyhow::anyhow!("proofs cannot be empty"));
        }
        if root.hash.is_empty() {
            return Err(anyhow::anyhow!("root hash cannot be empty"));
        }
        let num = self.proofs.len();
        let ics23_specs = Vec::<ics23::ProofSpec>::from(specs.clone());
        if ics23_specs.len() != num {
            return Err(anyhow::anyhow!(
                "number of specs does not match number of proofs"
            ));
        }
        if keys.key_path.len() != num {
            return Err(anyhow::anyhow!(
                "number of keys does not match number of proofs"
            ));
        }
        if value.is_empty() {
            return Err(anyhow::anyhow!("value cannot be empty"));
        }

        let mut subroot = value.clone();
        let mut value = value;
        // keys are represented from root-to-leaf
        for ((proof, spec), key) in self
            .proofs
            .iter()
            .zip(ics23_specs.iter())
            .zip(keys.key_path.iter().rev())
            .skip(start_index)
        {
            match &proof.proof {
                Some(Proof::Exist(existence_proof)) => {
                    subroot =
                        calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof)
                            .map_err(|_| anyhow::anyhow!("invalid merkle proof"))?;

                    if !verify_membership::<ics23::HostFunctionsManager>(
                        proof,
                        spec,
                        &subroot,
                        key.as_bytes(),
                        &value,
                    ) {
                        return Err(anyhow::anyhow!("merkle proof verification failed"));
                    }
                    value = subroot.clone();
                }
                _ => return Err(anyhow::anyhow!("invalid merkle proof")),
            }
        }

        if root.hash != subroot {
            return Err(anyhow::anyhow!(
                "merkle proof verification failed: root hash does not match"
            ));
        }

        Ok(())
    }

    pub fn verify_non_membership(
        &self,
        specs: &[ics23::ProofSpec],
        root: MerkleRoot,
        keys: MerklePath,
    ) -> Result<(), anyhow::Error> {
        // validate arguments
        if self.proofs.is_empty() {
            return Err(anyhow::anyhow!("proofs cannot be empty"));
        }
        if root.hash.is_empty() {
            return Err(anyhow::anyhow!("root hash cannot be empty"));
        }
        let num = self.proofs.len();
        let ics23_specs = Vec::<ics23::ProofSpec>::from(specs.clone());
        if ics23_specs.len() != num {
            return Err(anyhow::anyhow!(
                "number of specs does not match number of proofs"
            ));
        }
        if keys.key_path.len() != num {
            return Err(anyhow::anyhow!(
                "number of keys does not match number of proofs"
            ));
        }

        // verify the absence of key in lowest subtree
        let proof = self
            .proofs
            .get(0)
            .ok_or(anyhow::anyhow!("invalid merkle proof"))?;
        let spec = ics23_specs
            .get(0)
            .ok_or(anyhow::anyhow!("invalid merkle proof"))?;
        // keys are represented from root-to-leaf
        let key = keys
            .key_path
            .get(num - 1)
            .ok_or(anyhow::anyhow!("invalid merkle proof"))?;
        match &proof.proof {
            Some(Proof::Nonexist(non_existence_proof)) => {
                let subroot = calculate_non_existence_root(non_existence_proof)?;

                if !verify_non_membership::<ics23::HostFunctionsManager>(
                    proof,
                    spec,
                    &subroot,
                    key.as_bytes(),
                ) {
                    return Err(anyhow::anyhow!("merkle proof verification failed"));
                }

                // verify membership proofs starting from index 1 with value = subroot
                self.verify_membership(specs, root, keys, subroot, 1)
            }
            _ => Err(anyhow::anyhow!("invalid merkle proof")),
        }
    }
}
