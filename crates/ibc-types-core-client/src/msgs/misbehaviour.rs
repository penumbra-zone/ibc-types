use crate::prelude::*;

use ibc_proto::{
    google::protobuf::Any as ProtoAny,
    ibc::core::client::v1::MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour,
};
use ibc_types_domain_type::{DomainType, TypeUrl};

use crate::{error::Error, ClientId};

/// A type of message that submits client misbehaviour proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgSubmitMisbehaviour {
    /// client unique identifier
    pub client_id: ClientId,
    /// misbehaviour used for freezing the light client
    pub misbehaviour: ProtoAny,
    /// signer address
    pub signer: String,
}

impl TypeUrl for MsgSubmitMisbehaviour {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.MsgSubmitMisbehaviour";
}

impl DomainType for MsgSubmitMisbehaviour {
    type Proto = RawMsgSubmitMisbehaviour;
}

impl TryFrom<RawMsgSubmitMisbehaviour> for MsgSubmitMisbehaviour {
    type Error = Error;

    #[allow(deprecated)]
    fn try_from(raw: RawMsgSubmitMisbehaviour) -> Result<Self, Self::Error> {
        let raw_misbehaviour = raw.misbehaviour.ok_or(Error::MissingRawMisbehaviour)?;

        Ok(MsgSubmitMisbehaviour {
            client_id: raw
                .client_id
                .parse()
                .map_err(Error::InvalidRawMisbehaviour)?,
            misbehaviour: raw_misbehaviour,
            signer: raw.signer,
        })
    }
}

impl From<MsgSubmitMisbehaviour> for RawMsgSubmitMisbehaviour {
    #[allow(deprecated)]
    fn from(ics_msg: MsgSubmitMisbehaviour) -> Self {
        RawMsgSubmitMisbehaviour {
            client_id: ics_msg.client_id.to_string(),
            misbehaviour: Some(ics_msg.misbehaviour),
            signer: ics_msg.signer,
        }
    }
}
