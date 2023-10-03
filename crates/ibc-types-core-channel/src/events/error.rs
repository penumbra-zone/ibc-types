use crate::{prelude::*, ChannelError};

use core::num::ParseIntError;

use displaydoc::Display;
use ibc_types_core_client::HeightParseError;
use ibc_types_identifier::IdentifierError;
use ibc_types_timestamp::ParseTimestampError;

/// An error while parsing an event.
#[derive(Debug, Display)]
pub enum Error {
    /// Wrong event type: expected {expected}
    WrongType {
        // The actual event type is intentionally not included in the error, so
        // that Error::WrongType doesn't allocate and is cheap to use for trial
        // deserialization (attempt parsing of each event type in turn, which is
        // then just as fast as matching over the event type)
        //
        // TODO: is this good?
        expected: &'static str,
    },
    /// Missing expected event attribute "{0}"
    MissingAttribute(&'static str),
    /// Unexpected event attribute "{0}"
    UnexpectedAttribute(String),
    /// Error parsing channel order in "{key}": {e}
    ParseChannelOrder { key: &'static str, e: ChannelError },
    /// Error parsing hex bytes in "{key}": {e}
    ParseHex {
        key: &'static str,
        e: subtle_encoding::Error,
    },
    /// Error parsing timeout timestamp value in "{key}": {e}
    ParseTimeoutTimestampValue { key: &'static str, e: ParseIntError },
    /// Error parsing timeout timestamp in "{key}": {e}
    ParseTimeoutTimestamp {
        key: &'static str,
        e: ParseTimestampError,
    },
    /// Error parsing timeout height in "{key}": {e}
    ParseTimeoutHeight {
        key: &'static str,
        e: HeightParseError,
    },
    /// Error parsing channel ID in "{key}": {e}
    ParseChannelId {
        key: &'static str,
        e: IdentifierError,
    },
    /// Error parsing port ID in "{key}": {e}
    ParsePortId {
        key: &'static str,
        e: IdentifierError,
    },
    /// Error parsing connection ID in "{key}": {e}
    ParseConnectionId {
        key: &'static str,
        e: IdentifierError,
    },
    /// Error parsing packet sequence in "{key}": {e}
    ParseSequence { key: &'static str, e: ChannelError },
    /// Two different encodings of the same packet data were supplied, but they don't match.
    MismatchedPacketData,
    /// Two different encodings of the same acknowledgements were supplied, but they don't match.
    MismatchedAcks,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Note: fill in if errors have causes
        match &self {
            Self::ParseChannelOrder { e, .. } => Some(e),
            Self::ParseHex { e, .. } => Some(e),
            _ => None,
        }
    }
}
