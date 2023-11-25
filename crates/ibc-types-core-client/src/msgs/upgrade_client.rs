//! Definition of domain type msg `MsgUpgradeAnyClient`.

use crate::prelude::*;

use core::str::FromStr;

use ibc_proto::{
    google::protobuf::Any,
    ibc::core::{
        client::v1::MsgUpgradeClient as RawMsgUpgradeClient,
        commitment::v1::MerkleProof as RawMerkleProof,
    },
};
use ibc_types_domain_type::{DomainType, TypeUrl};
use prost::Message;

use crate::{error::Error, ClientId};

/// A type of message that triggers the upgrade of an on-chain (IBC) client.
#[derive(Clone, Debug, PartialEq)]
pub struct MsgUpgradeClient {
    // client unique identifier
    pub client_id: ClientId,
    // Upgraded client state
    pub client_state: Any,
    // Upgraded consensus state, only contains enough information
    // to serve as a basis of trust in update logic
    pub consensus_state: Any,
    // proof that old chain committed to new client
    pub proof_upgrade_client: RawMerkleProof,
    // proof that old chain committed to new consensus state
    pub proof_upgrade_consensus_state: RawMerkleProof,
    // signer address
    pub signer: String,
}


impl DomainType for MsgUpgradeClient {
    type Proto = RawMsgUpgradeClient;
}

impl From<MsgUpgradeClient> for RawMsgUpgradeClient {
    fn from(dm_msg: MsgUpgradeClient) -> RawMsgUpgradeClient {
        RawMsgUpgradeClient {
            client_id: dm_msg.client_id.to_string(),
            client_state: Some(dm_msg.client_state),
            consensus_state: Some(dm_msg.consensus_state),
            proof_upgrade_client: dm_msg.proof_upgrade_client.encode_to_vec(),
            proof_upgrade_consensus_state: dm_msg.proof_upgrade_consensus_state.encode_to_vec(),
            signer: dm_msg.signer,
        }
    }
}

impl TryFrom<RawMsgUpgradeClient> for MsgUpgradeClient {
    type Error = Error;

    fn try_from(proto_msg: RawMsgUpgradeClient) -> Result<Self, Self::Error> {
        let raw_client_state = proto_msg.client_state.ok_or(Error::MissingRawClientState)?;

        let raw_consensus_state = proto_msg
            .consensus_state
            .ok_or(Error::MissingRawConsensusState)?;

        Ok(MsgUpgradeClient {
            client_id: ClientId::from_str(&proto_msg.client_id)
                .map_err(Error::InvalidClientIdentifier)?,
            client_state: raw_client_state,
            consensus_state: raw_consensus_state,
            proof_upgrade_client: RawMerkleProof::decode(proto_msg.proof_upgrade_client.as_ref())
                .map_err(Error::InvalidUpgradeClientProof)?,
            proof_upgrade_consensus_state: RawMerkleProof::decode(
                proto_msg.proof_upgrade_consensus_state.as_ref(),
            )
            .map_err(Error::InvalidUpgradeConsensusStateProof)?,
            signer: proto_msg.signer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Height;

    use crate::mock::{
        client_state::MockClientState, consensus_state::MockConsensusState, header::MockHeader,
    };

    /// Returns a dummy `RawMerkleProof`, for testing only!
    pub fn get_dummy_merkle_proof() -> RawMerkleProof {
        use ibc_proto::ics23::CommitmentProof;

        let parsed = CommitmentProof { proof: None };
        let mproofs: vec::Vec<CommitmentProof> = vec![parsed];
        RawMerkleProof { proofs: mproofs }
    }

    /*
       pub fn get_dummy_proof() -> Vec<u8> {
           "Y29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIy"
               .as_bytes()
               .to_vec()
       }

       /// Returns a dummy `RawMsgUpgradeClient`, for testing only!
       pub fn get_dummy_raw_msg_upgrade_client(height: Height) -> RawMsgUpgradeClient {
           RawMsgUpgradeClient {
               client_id: "tendermint".parse().unwrap(),
               client_state: Some(MockClientState::new(MockHeader::new(height)).into()),
               consensus_state: Some(MockConsensusState::new(MockHeader::new(height)).into()),
               proof_upgrade_client: get_dummy_proof(),
               proof_upgrade_consensus_state: get_dummy_proof(),
               signer: "dummy_signer".to_string(),
           }
       }
    */

    #[test]
    fn msg_upgrade_client_serialization() {
        let client_id: ClientId = "tendermint".parse().unwrap();
        let signer = "dummy_signer".to_string();

        let height = Height::new(1, 1).unwrap();

        let client_state = MockClientState::new(MockHeader::new(height));
        let consensus_state = MockConsensusState::new(MockHeader::new(height));

        let proof = get_dummy_merkle_proof();

        let msg = MsgUpgradeClient {
            client_id,
            client_state: client_state.into(),
            consensus_state: consensus_state.into(),
            proof_upgrade_client: proof.clone(),
            proof_upgrade_consensus_state: proof,
            signer,
        };

        let raw: RawMsgUpgradeClient = RawMsgUpgradeClient::from(msg.clone());
        let msg_back = MsgUpgradeClient::try_from(raw.clone()).unwrap();
        let raw_back: RawMsgUpgradeClient = RawMsgUpgradeClient::from(msg_back.clone());
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }
}
