use ibc_types_core_commitment::MerkleProof;
use ibc_types_domain_type::DomainType;
use ics23::CommitmentProof;

use crate::prelude::*;

pub fn get_dummy_proof() -> Vec<u8> {
    let m = MerkleProof {
        proofs: vec![CommitmentProof::default()],
    };
    m.encode_to_vec()
}

pub fn get_dummy_account_id() -> String {
    "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C".parse().unwrap()
}

pub fn get_dummy_bech32_account() -> String {
    "cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng".to_string()
}
