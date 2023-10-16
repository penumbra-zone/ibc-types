use crate::prelude::*;

use ibc_proto::{
    google::protobuf::Any,
    ibc::core::connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck,
};
use ibc_types_core_client::Height;
use ibc_types_core_commitment::MerkleProof;
use ibc_types_domain_type::{DomainType, TypeUrl};

use crate::{ConnectionError, ConnectionId, Version};

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of chain B.
#[derive(Clone, Debug, PartialEq)]
pub struct MsgConnectionOpenAck {
    /// ConnectionId that chain A has chosen for it's ConnectionEnd
    pub conn_id_on_a: ConnectionId,
    /// ConnectionId that chain B has chosen for it's ConnectionEnd
    pub conn_id_on_b: ConnectionId,
    /// ClientState of client tracking chain A on chain B
    pub client_state_of_a_on_b: Any,
    /// proof of ConnectionEnd stored on Chain B during ConnOpenTry
    pub proof_conn_end_on_b: MerkleProof,
    /// proof of ClientState tracking chain A on chain B
    pub proof_client_state_of_a_on_b: MerkleProof,
    /// proof that chain B has stored ConsensusState of chain A on its client
    pub proof_consensus_state_of_a_on_b: MerkleProof,
    /// Height at which all proofs in this message were taken
    pub proofs_height_on_b: Height,
    /// height of latest header of chain A that updated the client on chain B
    pub consensus_height_of_a_on_b: Height,
    /// optional proof of the consensus state of the host chain, see: https://github.com/cosmos/ibc/pull/839
    host_consensus_state_proof: Option<MerkleProof>,
    pub version: Version,
    pub signer: String,
}

impl TypeUrl for MsgConnectionOpenAck {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenAck";
}

impl DomainType for MsgConnectionOpenAck {
    type Proto = RawMsgConnectionOpenAck;
}

impl TryFrom<RawMsgConnectionOpenAck> for MsgConnectionOpenAck {
    type Error = ConnectionError;

    fn try_from(raw: RawMsgConnectionOpenAck) -> Result<Self, Self::Error> {
        Ok(Self {
            conn_id_on_a: raw
                .connection_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            conn_id_on_b: raw
                .counterparty_connection_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            client_state_of_a_on_b: raw
                .client_state
                .ok_or(ConnectionError::MissingClientState)?,
            version: raw
                .version
                .ok_or(ConnectionError::EmptyVersions)?
                .try_into()?,
            proof_conn_end_on_b: MerkleProof::decode(raw.proof_try.as_ref())
                .map_err(|_| ConnectionError::InvalidProof)?,
            proof_client_state_of_a_on_b: MerkleProof::decode(raw.proof_client.as_ref())
                .map_err(|_| ConnectionError::InvalidProof)?,
            proof_consensus_state_of_a_on_b: MerkleProof::decode(raw.proof_consensus.as_ref())
                .map_err(|_| ConnectionError::InvalidProof)?,
            proofs_height_on_b: raw
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ConnectionError::MissingProofHeight)?,
            consensus_height_of_a_on_b: raw
                .consensus_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ConnectionError::MissingConsensusHeight)?,
            signer: raw.signer,
            host_consensus_state_proof: if raw.host_consensus_state_proof.is_empty() {
                None
            } else {
                Some(
                    MerkleProof::decode(raw.host_consensus_state_proof.as_ref())
                        .map_err(|_| ConnectionError::InvalidProof)?,
                )
            },
        })
    }
}

impl From<MsgConnectionOpenAck> for RawMsgConnectionOpenAck {
    fn from(msg: MsgConnectionOpenAck) -> Self {
        RawMsgConnectionOpenAck {
            connection_id: msg.conn_id_on_a.as_str().to_string(),
            counterparty_connection_id: msg.conn_id_on_b.as_str().to_string(),
            client_state: Some(msg.client_state_of_a_on_b),
            proof_height: Some(msg.proofs_height_on_b.into()),
            proof_try: msg.proof_conn_end_on_b.encode_to_vec(),
            proof_client: msg.proof_client_state_of_a_on_b.encode_to_vec(),
            proof_consensus: msg.proof_consensus_state_of_a_on_b.encode_to_vec(),
            host_consensus_state_proof: match msg.host_consensus_state_proof {
                Some(proof) => proof.encode_to_vec(),
                None => vec![],
            },
            consensus_height: Some(msg.consensus_height_of_a_on_b.into()),
            version: Some(msg.version.into()),
            signer: msg.signer,
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::prelude::*;
    use ibc_proto::ibc::core::{
        client::v1::Height as RawHeight,
        connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck,
    };

    use ibc_types_core_client::{
        mock::{client_state::MockClientState, header::MockHeader},
        Height,
    };

    use crate::{
        msgs::test_util::{get_dummy_bech32_account, get_dummy_proof},
        ConnectionId, Version,
    };

    use super::MsgConnectionOpenAck;

    /// Testing-specific helper methods.
    impl MsgConnectionOpenAck {
        /// Returns a new `MsgConnectionOpenAck` with dummy values.
        pub fn new_dummy(proof_height: u64, consensus_height: u64) -> Self {
            MsgConnectionOpenAck::try_from(get_dummy_raw_msg_conn_open_ack(
                proof_height,
                consensus_height,
            ))
            .unwrap()
        }
    }

    pub fn get_dummy_raw_msg_conn_open_ack(
        proof_height: u64,
        consensus_height: u64,
    ) -> RawMsgConnectionOpenAck {
        let client_state_height = Height::new(0, consensus_height).unwrap();
        RawMsgConnectionOpenAck {
            connection_id: ConnectionId::new(0).to_string(),
            counterparty_connection_id: ConnectionId::new(1).to_string(),
            proof_try: get_dummy_proof(),
            proof_height: Some(RawHeight {
                revision_number: 0,
                revision_height: proof_height,
            }),
            proof_consensus: get_dummy_proof(),
            consensus_height: Some(RawHeight {
                revision_number: 0,
                revision_height: consensus_height,
            }),
            client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
            proof_client: get_dummy_proof(),
            version: Some(Version::default().into()),
            signer: get_dummy_bech32_account(),
            host_consensus_state_proof: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::{
        client::v1::Height, connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck,
    };

    use super::*;

    #[test]
    fn parse_connection_open_ack_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenAck,
            want_pass: bool,
        }

        let default_ack_msg = test_util::get_dummy_raw_msg_conn_open_ack(5, 5);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_ack_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Bad connection id, non-alpha".to_string(),
                raw: RawMsgConnectionOpenAck {
                    connection_id: "con007".to_string(),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad version, missing version".to_string(),
                raw: RawMsgConnectionOpenAck {
                    version: None,
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad proof height, height is 0".to_string(),
                raw: RawMsgConnectionOpenAck {
                    proof_height: Some(Height {
                        revision_number: 1,
                        revision_height: 0,
                    }),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad consensus height, height is 0".to_string(),
                raw: RawMsgConnectionOpenAck {
                    consensus_height: Some(Height {
                        revision_number: 1,
                        revision_height: 0,
                    }),
                    ..default_ack_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let msg = MsgConnectionOpenAck::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenAck::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = test_util::get_dummy_raw_msg_conn_open_ack(5, 6);
        let msg = MsgConnectionOpenAck::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenAck::from(msg.clone());
        let msg_back = MsgConnectionOpenAck::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
