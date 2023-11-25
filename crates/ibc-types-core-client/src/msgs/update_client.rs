//! Definition of domain type message `MsgUpdateAnyClient`.

use crate::prelude::*;

use ibc_proto::{
    google::protobuf::Any, ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient,
};
use ibc_types_domain_type::{DomainType, TypeUrl};

use crate::{error::Error, ClientId};

/// A type of message that triggers the update of an on-chain (IBC) client with new headers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Any,
    pub signer: String,
}


impl DomainType for MsgUpdateClient {
    type Proto = RawMsgUpdateClient;
}

impl TryFrom<RawMsgUpdateClient> for MsgUpdateClient {
    type Error = Error;

    fn try_from(raw: RawMsgUpdateClient) -> Result<Self, Self::Error> {
        Ok(MsgUpdateClient {
            client_id: raw
                .client_id
                .parse()
                .map_err(Error::InvalidMsgUpdateClientId)?,
            client_message: raw.client_message.ok_or(Error::MissingRawClientMessage)?,
            signer: raw.signer,
        })
    }
}

impl From<MsgUpdateClient> for RawMsgUpdateClient {
    fn from(ics_msg: MsgUpdateClient) -> Self {
        RawMsgUpdateClient {
            client_id: ics_msg.client_id.to_string(),
            client_message: Some(ics_msg.client_message),
            signer: ics_msg.signer,
        }
    }
}
