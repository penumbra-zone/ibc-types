//! Definition of domain type message `MsgCreateClient`.

use crate::prelude::*;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::MsgCreateClient as RawMsgCreateClient;
use ibc_types_domain_type::{DomainType, TypeUrl};

use crate::error::ClientError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgCreateClient {
    pub client_state: Any,
    pub consensus_state: Any,
    pub signer: String,
}

impl TypeUrl for MsgCreateClient {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.MsgCreateClient";
}

impl DomainType for MsgCreateClient {
    type Proto = RawMsgCreateClient;
}

impl TryFrom<RawMsgCreateClient> for MsgCreateClient {
    type Error = ClientError;

    fn try_from(raw: RawMsgCreateClient) -> Result<Self, Self::Error> {
        let client_state = raw.client_state.ok_or(ClientError::MissingRawClientState)?;

        let consensus_state = raw
            .consensus_state
            .ok_or(ClientError::MissingRawConsensusState)?;

        Ok(MsgCreateClient {
            client_state,
            consensus_state,
            signer: raw.signer,
        })
    }
}

impl From<MsgCreateClient> for RawMsgCreateClient {
    fn from(ics_msg: MsgCreateClient) -> Self {
        RawMsgCreateClient {
            client_state: Some(ics_msg.client_state),
            consensus_state: Some(ics_msg.consensus_state),
            signer: ics_msg.signer,
        }
    }
}
