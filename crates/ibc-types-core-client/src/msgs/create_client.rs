//! Definition of domain type message `MsgCreateClient`.

use crate::prelude::*;

use ibc_proto::{
    google::protobuf::Any, ibc::core::client::v1::MsgCreateClient as RawMsgCreateClient,
};
use ibc_types_domain_type::DomainType;

use crate::error::Error;

/// A type of message that triggers the creation of a new on-chain (IBC) client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgCreateClient {
    pub client_state: Any,
    pub consensus_state: Any,
    pub signer: String,
}

impl DomainType for MsgCreateClient {
    type Proto = RawMsgCreateClient;
}

impl TryFrom<RawMsgCreateClient> for MsgCreateClient {
    type Error = Error;

    fn try_from(raw: RawMsgCreateClient) -> Result<Self, Self::Error> {
        let client_state = raw.client_state.ok_or(Error::MissingRawClientState)?;

        let consensus_state = raw.consensus_state.ok_or(Error::MissingRawConsensusState)?;

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
