use crate::prelude::*;

use core::time::Duration;
use core::{
    fmt::{Display, Error as FmtError, Formatter},
    u64,
};

use ibc_proto::ibc::core::connection::v1::{
    ConnectionEnd as RawConnectionEnd, Counterparty as RawCounterparty,
    IdentifiedConnection as RawIdentifiedConnection,
};
use ibc_proto::protobuf::Protobuf;

use ibc_types_core_client::ClientId;
use ibc_types_domain_type::{DomainType, TypeUrl};
use ibc_types_timestamp::ZERO_DURATION;

use crate::{ConnectionError, ConnectionId, Version};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdentifiedConnectionEnd {
    pub connection_id: ConnectionId,
    pub connection_end: ConnectionEnd,
}

impl IdentifiedConnectionEnd {
    pub fn new(connection_id: ConnectionId, connection_end: ConnectionEnd) -> Self {
        IdentifiedConnectionEnd {
            connection_id,
            connection_end,
        }
    }

    pub fn id(&self) -> &ConnectionId {
        &self.connection_id
    }

    pub fn end(&self) -> &ConnectionEnd {
        &self.connection_end
    }
}

impl DomainType for IdentifiedConnectionEnd {
    type Proto = RawIdentifiedConnection;
}
impl TypeUrl for IdentifiedConnectionEnd {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.IdentifiedConnection";
}

impl TryFrom<RawIdentifiedConnection> for IdentifiedConnectionEnd {
    type Error = ConnectionError;

    fn try_from(value: RawIdentifiedConnection) -> Result<Self, Self::Error> {
        let raw_connection_end = RawConnectionEnd {
            client_id: value.client_id.to_string(),
            versions: value.versions,
            state: value.state,
            counterparty: value.counterparty,
            delay_period: value.delay_period,
        };

        Ok(IdentifiedConnectionEnd {
            connection_id: value
                .id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            connection_end: raw_connection_end.try_into()?,
        })
    }
}

impl From<IdentifiedConnectionEnd> for RawIdentifiedConnection {
    fn from(value: IdentifiedConnectionEnd) -> Self {
        RawIdentifiedConnection {
            id: value.connection_id.to_string(),
            client_id: value.connection_end.client_id.to_string(),
            versions: value
                .connection_end
                .versions
                .iter()
                .map(|v| From::from(v.clone()))
                .collect(),
            state: value.connection_end.state as i32,
            delay_period: value.connection_end.delay_period.as_nanos() as u64,
            counterparty: Some(value.connection_end.counterparty.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionEnd {
    pub state: State,
    pub client_id: ClientId,
    pub counterparty: Counterparty,
    pub versions: Vec<Version>,
    pub delay_period: Duration,
}

impl Default for ConnectionEnd {
    fn default() -> Self {
        Self {
            state: State::Uninitialized,
            client_id: Default::default(),
            counterparty: Default::default(),
            versions: Vec::new(),
            delay_period: ZERO_DURATION,
        }
    }
}

impl Protobuf<RawConnectionEnd> for ConnectionEnd {}

impl TryFrom<RawConnectionEnd> for ConnectionEnd {
    type Error = ConnectionError;
    fn try_from(value: RawConnectionEnd) -> Result<Self, Self::Error> {
        let state = value.state.try_into()?;
        if state == State::Uninitialized {
            return Ok(ConnectionEnd::default());
        }
        if value.client_id.is_empty() {
            return Err(ConnectionError::EmptyProtoConnectionEnd);
        }

        Ok(Self {
            state,
            client_id: value
                .client_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            counterparty: value
                .counterparty
                .ok_or(ConnectionError::MissingCounterparty)?
                .try_into()?,
            versions: value
                .versions
                .into_iter()
                .map(Version::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            delay_period: Duration::from_nanos(value.delay_period),
        })
    }
}

impl From<ConnectionEnd> for RawConnectionEnd {
    fn from(value: ConnectionEnd) -> Self {
        RawConnectionEnd {
            client_id: value.client_id.to_string(),
            versions: value
                .versions
                .iter()
                .map(|v| From::from(v.clone()))
                .collect(),
            state: value.state as i32,
            counterparty: Some(value.counterparty.into()),
            delay_period: value.delay_period.as_nanos() as u64,
        }
    }
}

impl ConnectionEnd {
    /// Helper function to compare the counterparty of this end with another counterparty.
    pub fn counterparty_matches(&self, other: &Counterparty) -> bool {
        self.counterparty.eq(other)
    }

    /// Helper function to compare the client id of this end with another client identifier.
    pub fn client_id_matches(&self, other: &ClientId) -> bool {
        self.client_id.eq(other)
    }

    /// Helper function to determine whether the connection is open.
    pub fn is_open(&self) -> bool {
        self.state_matches(&State::Open)
    }

    /// Helper function to determine whether the connection is uninitialized.
    pub fn is_uninitialized(&self) -> bool {
        self.state_matches(&State::Uninitialized)
    }

    /// Helper function to compare the state of this end with another state.
    pub fn state_matches(&self, other: &State) -> bool {
        self.state.eq(other)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Counterparty {
    pub client_id: ClientId,
    pub connection_id: Option<ConnectionId>,
    pub prefix: Vec<u8>,
}

impl Protobuf<RawCounterparty> for Counterparty {}

// Converts from the wire format RawCounterparty. Typically used from the relayer side
// during queries for response validation and to extract the Counterparty structure.
impl TryFrom<RawCounterparty> for Counterparty {
    type Error = ConnectionError;

    fn try_from(raw_counterparty: RawCounterparty) -> Result<Self, Self::Error> {
        let connection_id: Option<ConnectionId> = if raw_counterparty.connection_id.is_empty() {
            None
        } else {
            Some(
                raw_counterparty
                    .connection_id
                    .parse()
                    .map_err(ConnectionError::InvalidIdentifier)?,
            )
        };
        Ok(Counterparty {
            client_id: raw_counterparty
                .client_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            connection_id,
            prefix: raw_counterparty
                .prefix
                .ok_or(ConnectionError::MissingCounterparty)?
                .key_prefix,
        })
    }
}

impl From<Counterparty> for RawCounterparty {
    fn from(value: Counterparty) -> Self {
        RawCounterparty {
            client_id: value.client_id.as_str().to_string(),
            connection_id: value
                .connection_id
                .map_or_else(|| "".to_string(), |v| v.as_str().to_string()),
            prefix: Some(ibc_proto::ibc::core::commitment::v1::MerklePrefix {
                key_prefix: value.prefix,
            }),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum State {
    Uninitialized = 0isize,
    Init = 1isize,
    TryOpen = 2isize,
    Open = 3isize,
}

impl State {
    /// Yields the State as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uninitialized => "UNINITIALIZED",
            Self::Init => "INIT",
            Self::TryOpen => "TRYOPEN",
            Self::Open => "OPEN",
        }
    }

    /// Parses the State out from a i32.
    pub fn from_i32(s: i32) -> Result<Self, ConnectionError> {
        match s {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            _ => Err(ConnectionError::InvalidState { state: s }),
        }
    }

    /// Returns whether or not this connection state is `Open`.
    pub fn is_open(self) -> bool {
        self == State::Open
    }

    /// Returns whether or not this connection with this state
    /// has progressed less or the same than the argument.
    ///
    /// # Example
    /// ```rust,ignore
    /// assert!(State::Init.less_or_equal_progress(State::Open));
    /// assert!(State::TryOpen.less_or_equal_progress(State::TryOpen));
    /// assert!(!State::Open.less_or_equal_progress(State::Uninitialized));
    /// ```
    pub fn less_or_equal_progress(self, other: Self) -> bool {
        self as u32 <= other as u32
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<i32> for State {
    type Error = ConnectionError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            _ => Err(ConnectionError::InvalidState { state: value }),
        }
    }
}

impl From<State> for i32 {
    fn from(value: State) -> Self {
        value.into()
    }
}
