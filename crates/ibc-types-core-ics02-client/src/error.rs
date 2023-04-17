use crate::prelude::*;

use displaydoc::Display;
use ibc_proto::protobuf::Error as TendermintProtoError;

use crate::client_type::ClientType;
use crate::height::Height;

#[derive(Debug, Display)]
pub enum ClientError {
    /// client is frozen with description: `{description}`
    ClientFrozen { description: String },
    /// implementation specific error
    ImplementationSpecific,
    /// header verification failed with reason: `{reason}`
    HeaderVerificationFailure { reason: String },
    /// failed to build trust threshold from fraction: `{numerator}`/`{denominator}`
    InvalidTrustThreshold { numerator: u64, denominator: u64 },
    /// failed to build Tendermint domain type trust threshold from fraction: `{numerator}`/`{denominator}`
    FailedTrustThresholdConversion { numerator: u64, denominator: u64 },
    /// unknown client state type: `{client_state_type}`
    UnknownClientStateType { client_state_type: String },
    /// empty prefix
    EmptyPrefix,
    /// unknown client consensus state type: `{consensus_state_type}`
    UnknownConsensusStateType { consensus_state_type: String },
    /// unknown header type: `{header_type}`
    UnknownHeaderType { header_type: String },
    /// unknown misbehaviour type: `{misbehaviour_type}`
    UnknownMisbehaviourType { misbehaviour_type: String },
    /// missing raw client state
    MissingRawClientState,
    /// missing raw client consensus state
    MissingRawConsensusState,
    /// decode error: `{0}`
    Decode(prost::DecodeError),
    /// invalid raw header error: `{0}`
    InvalidRawHeader(TendermintProtoError),
    /// missing raw header
    MissingRawHeader,
    /// missing raw misbehaviour
    MissingRawMisbehaviour,
    /// revision height cannot be zero
    InvalidHeight,
    /// height cannot end up zero or negative
    InvalidHeightResult,
    /// the proof height is insufficient: latest_height=`{latest_height}` proof_height=`{proof_height}`
    InvalidProofHeight {
        latest_height: Height,
        proof_height: Height,
    },
    /// mismatch between client and arguments types
    ClientArgsTypeMismatch { client_type: ClientType },
    /// received header height (`{header_height}`) is lower than (or equal to) client latest height (`{latest_height}`)
    LowHeaderHeight {
        header_height: Height,
        latest_height: Height,
    },
    /// upgraded client height `{upgraded_height}` must be at greater than current client height `{client_height}`
    LowUpgradeHeight {
        upgraded_height: Height,
        client_height: Height,
    },
    /// the local consensus state could not be retrieved for height `{height}`
    MissingLocalConsensusState { height: Height },
    /// misbehaviour handling failed with reason: `{reason}`
    MisbehaviourHandlingFailure { reason: String },
    /// client specific error: `{description}`
    ClientSpecific { description: String },
    /// other error: `{description}`
    Other { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ClientIdentifierConstructor {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidMsgUpdateClientId(e) => Some(e),
            Self::InvalidClientIdentifier(e) => Some(e),
            Self::InvalidRawHeader(e) => Some(e),
            Self::InvalidRawMisbehaviour(e) => Some(e),
            Self::InvalidUpgradeClientProof(e) => Some(e),
            Self::InvalidUpgradeConsensusStateProof(e) => Some(e),
            Self::InvalidCommitmentProof(e) => Some(e),
            Self::Signer(e) => Some(e),
            Self::Ics23Verification(e) => Some(e),
            _ => None,
        }
    }
}
