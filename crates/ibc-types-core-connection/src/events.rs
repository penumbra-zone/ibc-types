//! Types for ABCI [`Event`](tendermint::abci::Event)s that inform relayers
//! about IBC connection events.

use core::str::FromStr;

use displaydoc::Display;
use ibc_types_core_client::ClientId;
use ibc_types_identifier::IdentifierError;
use tendermint::{
    abci,
    abci::{Event, TypedEvent},
};

use crate::prelude::*;
use crate::ConnectionId;

/// An error while parsing an [`Event`].
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
    /// Error parsing connection ID in "{key}": {e}
    ParseConnectionId {
        key: &'static str,
        e: IdentifierError,
    },
    /// Error parsing client ID in "{key}": {e}
    ParseClientId {
        key: &'static str,
        e: IdentifierError,
    },
    /// Error parsing hex bytes in "{key}": {e}
    ParseHex {
        key: &'static str,
        e: subtle_encoding::Error,
    },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Note: fill in if errors have causes
        match &self {
            Self::ParseConnectionId { e, .. } => Some(e),
            _ => None,
        }
    }
}

/// Common attributes for IBC connection events.
///
/// This is an internal type only used to commonize (de)serialization code.
struct Attributes {
    connection_id: ConnectionId,
    client_id: ClientId,
    counterparty_connection_id: Option<ConnectionId>,
    counterparty_client_id: ClientId,
}

/// Convert attributes to Tendermint ABCI tags
impl From<Attributes> for Vec<abci::EventAttribute> {
    fn from(a: Attributes) -> Self {
        let conn_id = ("connection_id", a.connection_id.as_str()).into();
        let client_id = ("client_id", a.client_id.as_str()).into();

        let counterparty_conn_id = (
            "counterparty_connection_id",
            a.counterparty_connection_id
                .as_ref()
                .map(|id| id.as_str())
                .unwrap_or(""),
        )
            .into();

        let counterparty_client_id =
            ("counterparty_client_id", a.counterparty_client_id.as_str()).into();

        vec![
            conn_id,
            client_id,
            counterparty_client_id,
            counterparty_conn_id,
        ]
    }
}

impl TryFrom<Vec<abci::EventAttribute>> for Attributes {
    type Error = Error;
    fn try_from(attributes: Vec<abci::EventAttribute>) -> Result<Self, Self::Error> {
        let mut client_id = None;
        let mut connection_id = None;
        let mut counterparty_client_id = None;
        let mut counterparty_connection_id = None;

        for attr in attributes {
            match attr.key.as_ref() {
                "connection_id" => {
                    connection_id =
                        Some(ConnectionId::from_str(attr.value.as_ref()).map_err(|e| {
                            Error::ParseConnectionId {
                                key: "connection_id",
                                e,
                            }
                        })?);
                }
                "client_id" => {
                    client_id = Some(ClientId::from_str(attr.value.as_ref()).map_err(|e| {
                        Error::ParseClientId {
                            key: "client_id",
                            e,
                        }
                    })?);
                }
                "counterparty_connection_id" => {
                    counterparty_connection_id = if attr.value.is_empty() {
                        // Don't try to parse the connection ID if it was empty; set it to
                        // None instead, since we'll reject empty connection IDs in parsing.
                        None
                    } else {
                        Some(ConnectionId::from_str(attr.value.as_ref()).map_err(|e| {
                            Error::ParseConnectionId {
                                key: "counterparty_connection_id",
                                e,
                            }
                        })?)
                    };
                }
                "counterparty_client_id" => {
                    counterparty_client_id =
                        Some(ClientId::from_str(attr.value.as_ref()).map_err(|e| {
                            Error::ParseClientId {
                                key: "counterparty_client_id",
                                e,
                            }
                        })?);
                }
                _ => return Err(Error::UnexpectedAttribute(attr.key)),
            }
        }

        Ok(Self {
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
            client_id: client_id.ok_or(Error::MissingAttribute("client_id"))?,
            counterparty_connection_id,
            counterparty_client_id: counterparty_client_id
                .ok_or(Error::MissingAttribute("counterparty_client_id"))?,
        })
    }
}

/// Per our convention, this event is generated on chain A.
pub struct ConnectionOpenInit {
    pub connection_id: ConnectionId,
    pub client_id_on_a: ClientId,
    pub client_id_on_b: ClientId,
}

impl ConnectionOpenInit {
    pub const TYPE_STR: &'static str = "connection_open_init";
}

impl TypedEvent for ConnectionOpenInit {}

impl From<ConnectionOpenInit> for Event {
    fn from(e: ConnectionOpenInit) -> Self {
        let attributes: Vec<abci::EventAttribute> = Attributes {
            connection_id: e.connection_id,
            client_id: e.client_id_on_a,
            counterparty_connection_id: None,
            counterparty_client_id: e.client_id_on_b,
        }
        .into();

        Event::new(ConnectionOpenInit::TYPE_STR, attributes)
    }
}

impl TryFrom<Event> for ConnectionOpenInit {
    type Error = Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != ConnectionOpenInit::TYPE_STR {
            return Err(Error::WrongType {
                expected: ConnectionOpenInit::TYPE_STR,
            });
        }

        let attributes = Attributes::try_from(event.attributes)?;

        Ok(Self {
            connection_id: attributes.connection_id,
            client_id_on_a: attributes.client_id,
            client_id_on_b: attributes.counterparty_client_id,
        })
    }
}

/// Per our convention, this event is generated on chain B.
pub struct ConnectionOpenTry {
    pub conn_id_on_b: ConnectionId,
    pub client_id_on_b: ClientId,
    pub conn_id_on_a: ConnectionId,
    pub client_id_on_a: ClientId,
}

impl ConnectionOpenTry {
    pub const TYPE_STR: &'static str = "connection_open_try";
}

impl TypedEvent for ConnectionOpenTry {}

impl From<ConnectionOpenTry> for Event {
    fn from(e: ConnectionOpenTry) -> Self {
        let attributes: Vec<abci::EventAttribute> = Attributes {
            connection_id: e.conn_id_on_b,
            client_id: e.client_id_on_b,
            counterparty_connection_id: Some(e.conn_id_on_a),
            counterparty_client_id: e.client_id_on_a,
        }
        .into();

        Event::new(ConnectionOpenTry::TYPE_STR, attributes)
    }
}

impl TryFrom<Event> for ConnectionOpenTry {
    type Error = Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != ConnectionOpenTry::TYPE_STR {
            return Err(Error::WrongType {
                expected: ConnectionOpenTry::TYPE_STR,
            });
        }

        let attributes = Attributes::try_from(event.attributes)?;

        Ok(Self {
            conn_id_on_b: attributes.connection_id,
            client_id_on_b: attributes.client_id,
            conn_id_on_a: attributes
                .counterparty_connection_id
                .ok_or(Error::MissingAttribute("counterparty_connection_id"))?,
            client_id_on_a: attributes.counterparty_client_id,
        })
    }
}

/// Per our convention, this event is generated on chain A.
pub struct ConnectionOpenAck {
    pub conn_id_on_a: ConnectionId,
    pub client_id_on_a: ClientId,
    pub conn_id_on_b: ConnectionId,
    pub client_id_on_b: ClientId,
}

impl ConnectionOpenAck {
    pub const TYPE_STR: &'static str = "connection_open_ack";
}

impl TypedEvent for ConnectionOpenAck {}

impl From<ConnectionOpenAck> for Event {
    fn from(e: ConnectionOpenAck) -> Self {
        let attributes: Vec<abci::EventAttribute> = Attributes {
            connection_id: e.conn_id_on_a,
            client_id: e.client_id_on_a,
            counterparty_connection_id: Some(e.conn_id_on_b),
            counterparty_client_id: e.client_id_on_b,
        }
        .into();

        Event::new(ConnectionOpenAck::TYPE_STR, attributes)
    }
}

impl TryFrom<Event> for ConnectionOpenAck {
    type Error = Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != ConnectionOpenAck::TYPE_STR {
            return Err(Error::WrongType {
                expected: ConnectionOpenAck::TYPE_STR,
            });
        }

        let attributes = Attributes::try_from(event.attributes)?;

        Ok(Self {
            conn_id_on_a: attributes.connection_id,
            client_id_on_a: attributes.client_id,
            conn_id_on_b: attributes
                .counterparty_connection_id
                .ok_or(Error::MissingAttribute("counterparty_connection_id"))?,
            client_id_on_b: attributes.counterparty_client_id,
        })
    }
}

/// Per our convention, this event is generated on chain B.
pub struct ConnectionOpenConfirm {
    pub conn_id_on_b: ConnectionId,
    pub client_id_on_b: ClientId,
    pub conn_id_on_a: ConnectionId,
    pub client_id_on_a: ClientId,
}

impl ConnectionOpenConfirm {
    pub const TYPE_STR: &'static str = "connection_open_confirm";
}

impl TypedEvent for ConnectionOpenConfirm {}

impl From<ConnectionOpenConfirm> for Event {
    fn from(e: ConnectionOpenConfirm) -> Self {
        let attributes: Vec<abci::EventAttribute> = Attributes {
            connection_id: e.conn_id_on_b,
            client_id: e.client_id_on_b,
            counterparty_connection_id: Some(e.conn_id_on_a),
            counterparty_client_id: e.client_id_on_a,
        }
        .into();

        Event::new(ConnectionOpenConfirm::TYPE_STR, attributes)
    }
}

impl TryFrom<Event> for ConnectionOpenConfirm {
    type Error = Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != ConnectionOpenConfirm::TYPE_STR {
            return Err(Error::WrongType {
                expected: ConnectionOpenConfirm::TYPE_STR,
            });
        }

        let attributes = Attributes::try_from(event.attributes)?;

        Ok(Self {
            conn_id_on_b: attributes.connection_id,
            client_id_on_b: attributes.client_id,
            conn_id_on_a: attributes
                .counterparty_connection_id
                .ok_or(Error::MissingAttribute("counterparty_connection_id"))?,
            client_id_on_a: attributes.counterparty_client_id,
        })
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ics02_client::client_type::ClientType;
    use tendermint::abci::Event as AbciEvent;

    #[test]
    fn ibc_to_abci_connection_events() {
        struct Test {
            kind: IbcEventType,
            event: AbciEvent,
            expected_keys: Vec<&'static str>,
            expected_values: Vec<&'static str>,
        }

        let client_type = ClientType::new("07-tendermint".to_string());
        let conn_id_on_a = ConnectionId::default();
        let client_id_on_a = ClientId::new(client_type.clone(), 0).unwrap();
        let conn_id_on_b = ConnectionId::new(1);
        let client_id_on_b = ClientId::new(client_type, 1).unwrap();
        let expected_keys = vec![
            "connection_id",
            "client_id",
            "counterparty_client_id",
            "counterparty_connection_id",
        ];
        let expected_values = vec![
            "connection-0",
            "07-tendermint-0",
            "07-tendermint-1",
            "connection-1",
        ];

        let tests: Vec<Test> = vec![
            Test {
                kind: IbcEventType::OpenInitConnection,
                event: OpenInit::new(
                    conn_id_on_a.clone(),
                    client_id_on_a.clone(),
                    client_id_on_b.clone(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| if i == 3 { "" } else { v })
                    .collect(),
            },
            Test {
                kind: IbcEventType::OpenTryConnection,
                event: OpenTry::new(
                    conn_id_on_b.clone(),
                    client_id_on_b.clone(),
                    conn_id_on_a.clone(),
                    client_id_on_a.clone(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values.iter().rev().cloned().collect(),
            },
            Test {
                kind: IbcEventType::OpenAckConnection,
                event: OpenAck::new(
                    conn_id_on_a.clone(),
                    client_id_on_a.clone(),
                    conn_id_on_b.clone(),
                    client_id_on_b.clone(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values.clone(),
            },
            Test {
                kind: IbcEventType::OpenConfirmConnection,
                event: OpenConfirm::new(conn_id_on_b, client_id_on_b, conn_id_on_a, client_id_on_a)
                    .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values.iter().rev().cloned().collect(),
            },
        ];

        for t in tests {
            assert_eq!(t.kind.as_str(), t.event.kind);
            assert_eq!(t.expected_keys.len(), t.event.attributes.len());
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.key,
                    t.expected_keys[i],
                    "key mismatch for {:?}",
                    t.kind.as_str()
                );
            }
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.value,
                    t.expected_values[i],
                    "value mismatch for {:?}",
                    t.kind.as_str()
                );
            }
        }
    }
}
*/
