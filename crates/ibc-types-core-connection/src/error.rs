use crate::ValidationError;
use crate::{ConnectionId, Version};
use ibc_types_core_client::ClientId;
use ibc_types_core_client::Error as ClientError;
use ibc_types_core_client::Height;

use alloc::string::String;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum ConnectionError {
    /// client error: `{0}`
    Client(ClientError),
    /// connection state is unknown: `{state}`
    InvalidState { state: i32 },
    /// connection end for identifier `{connection_id}` was never initialized
    ConnectionMismatch { connection_id: ConnectionId },
    /// consensus height claimed by the client on the other party is too advanced: `{target_height}` (host chain current height: `{current_height}`)
    InvalidConsensusHeight {
        target_height: Height,
        current_height: Height,
    },
    /// identifier error: `{0}`
    InvalidIdentifier(ValidationError),
    /// ConnectionEnd domain object could not be constructed out of empty proto object
    EmptyProtoConnectionEnd,
    /// empty supported versions
    EmptyVersions,
    /// empty supported features
    EmptyFeatures,
    /// no common version
    NoCommonVersion,
    /// version \"`{version}`\" not supported
    VersionNotSupported { version: Version },
    /// missing proof height
    MissingProofHeight,
    /// missing consensus height
    MissingConsensusHeight,
    /// invalid connection proof error
    InvalidProof,
    /// verifying connnection state error: `{0}`
    VerifyConnectionState(ClientError),
    /// malformed signer
    Signer,
    /// no connection was found for the previous connection id provided `{connection_id}`
    ConnectionNotFound { connection_id: ConnectionId },
    /// invalid counterparty
    InvalidCounterparty,
    /// missing counterparty
    MissingCounterparty,
    /// missing client state
    MissingClientState,
    /// the consensus proof verification failed (height: `{height}`), client error: `{client_error}`
    ConsensusStateVerificationFailure {
        height: Height,
        client_error: ClientError,
    },
    /// the client state proof verification failed for client id `{client_id}`, client error: `{client_error}`
    ClientStateVerificationFailure {
        // TODO: use more specific error source
        client_id: ClientId,
        client_error: ClientError,
    },
    /// invalid client state: `{reason}`
    InvalidClientState { reason: String },
    /// other error: `{description}`
    Other { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Client(e) => Some(e),
            Self::InvalidIdentifier(e) => Some(e),
            Self::VerifyConnectionState(e) => Some(e),
            Self::ConsensusStateVerificationFailure {
                client_error: e, ..
            } => Some(e),
            Self::ClientStateVerificationFailure {
                client_error: e, ..
            } => Some(e),
            _ => None,
        }
    }
}
