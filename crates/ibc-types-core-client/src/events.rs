//! Types for ABCI [`Event`]s that inform relayers about IBC client events.

use displaydoc::Display;
use subtle_encoding::hex;
use tendermint::{
    abci,
    abci::{Event, TypedEvent},
};

use crate::{
    client_type::ClientType,
    height::{Height, HeightParseError},
    prelude::*,
    ClientId,
};

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
    /// Error parsing height in "{key}": {e}
    ParseHeight {
        key: &'static str,
        e: HeightParseError,
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
            Self::ParseHeight { e, .. } => Some(e),
            _ => None,
        }
    }
}

/// CreateClient event signals the creation of a new on-chain client (IBC client).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateClient {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub consensus_height: Height,
}

impl CreateClient {
    pub const TYPE_STR: &'static str = "create_client";
}

impl TypedEvent for CreateClient {}

impl From<CreateClient> for abci::Event {
    fn from(c: CreateClient) -> Self {
        Event::new(
            CreateClient::TYPE_STR,
            [
                ("client_id", c.client_id.0),
                ("client_type", c.client_type.0),
                ("consensus_height", c.consensus_height.to_string()),
            ],
        )
    }
}

impl TryFrom<Event> for CreateClient {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != CreateClient::TYPE_STR {
            return Err(Error::WrongType {
                expected: CreateClient::TYPE_STR,
            });
        }

        let mut client_id = None;
        let mut client_type = None;
        let mut consensus_height = None;

        for attr in event.attributes {
            match attr.key.as_ref() {
                "client_id" => {
                    client_id = Some(ClientId(attr.value));
                }
                "client_type" => {
                    client_type = Some(ClientType(attr.value));
                }
                "consensus_height" => {
                    consensus_height =
                        Some(attr.value.parse().map_err(|e| Error::ParseHeight {
                            key: "consensus_height",
                            e,
                        })?);
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            client_id: client_id.ok_or(Error::MissingAttribute("client_id"))?,
            client_type: client_type.ok_or(Error::MissingAttribute("client_type"))?,
            consensus_height: consensus_height
                .ok_or(Error::MissingAttribute("consensus_height"))?,
        })
    }
}

/// UpdateClient event signals a recent update of an on-chain client (IBC Client).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateClient {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub consensus_height: Height,
    /// This can't be an Any because we don't have a type URL.
    pub header: Vec<u8>,
}

impl UpdateClient {
    pub const TYPE_STR: &'static str = "update_client";
}

impl TypedEvent for UpdateClient {}

impl From<UpdateClient> for abci::Event {
    fn from(u: UpdateClient) -> Self {
        Event::new(
            UpdateClient::TYPE_STR,
            [
                ("client_id", u.client_id.0),
                ("client_type", u.client_type.0),
                ("consensus_height", u.consensus_height.to_string()),
                ("header", String::from_utf8(hex::encode(u.header)).unwrap()),
            ],
        )
    }
}

impl TryFrom<Event> for UpdateClient {
    type Error = Error;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        if value.kind != UpdateClient::TYPE_STR {
            return Err(Error::WrongType {
                expected: UpdateClient::TYPE_STR,
            });
        }

        let mut client_id = None;
        let mut client_type = None;
        let mut consensus_height = None;
        let mut header = None;

        for attr in value.attributes {
            match attr.key.as_ref() {
                "client_id" => {
                    client_id = Some(ClientId(attr.value));
                }
                "client_type" => {
                    client_type = Some(ClientType(attr.value));
                }
                "consensus_height" => {
                    consensus_height =
                        Some(attr.value.parse().map_err(|e| Error::ParseHeight {
                            key: "consensus_height",
                            e,
                        })?);
                }
                "header" => {
                    header = Some(
                        hex::decode(attr.value)
                            .map_err(|e| Error::ParseHex { key: "header", e })?,
                    );
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            client_id: client_id.ok_or(Error::MissingAttribute("client_id"))?,
            client_type: client_type.ok_or(Error::MissingAttribute("client_type"))?,
            consensus_height: consensus_height
                .ok_or(Error::MissingAttribute("consensus_height"))?,
            header: header.ok_or(Error::MissingAttribute("header"))?,
        })
    }
}

/// ClientMisbehaviour event signals the update of an on-chain client (IBC Client) with evidence of
/// misbehaviour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientMisbehaviour {
    pub client_id: ClientId,
    pub client_type: ClientType,
}

impl ClientMisbehaviour {
    pub const TYPE_STR: &'static str = "client_misbehaviour";
}

impl From<ClientMisbehaviour> for abci::Event {
    fn from(c: ClientMisbehaviour) -> Self {
        Event::new(
            ClientMisbehaviour::TYPE_STR,
            [
                ("client_id", c.client_id.0),
                ("client_type", c.client_type.0),
            ],
        )
    }
}

impl TryFrom<Event> for ClientMisbehaviour {
    type Error = Error;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        if value.kind != ClientMisbehaviour::TYPE_STR {
            return Err(Error::WrongType {
                expected: ClientMisbehaviour::TYPE_STR,
            });
        }

        let mut client_id = None;
        let mut client_type = None;

        for attr in value.attributes {
            match attr.key.as_ref() {
                "client_id" => {
                    client_id = Some(ClientId(attr.value));
                }
                "client_type" => {
                    client_type = Some(ClientType(attr.value));
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            client_id: client_id.ok_or(Error::MissingAttribute("client_id"))?,
            client_type: client_type.ok_or(Error::MissingAttribute("client_type"))?,
        })
    }
}

/// Signals a recent upgrade of an on-chain client (IBC Client).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeClient {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub consensus_height: Height,
}

impl UpgradeClient {
    pub const TYPE_STR: &'static str = "upgrade_client";
}

impl From<UpgradeClient> for abci::Event {
    fn from(u: UpgradeClient) -> Self {
        Event::new(
            UpgradeClient::TYPE_STR,
            [
                ("client_id", u.client_id.0),
                ("client_type", u.client_type.0),
                ("consensus_height", u.consensus_height.to_string()),
            ],
        )
    }
}

impl TryFrom<Event> for UpgradeClient {
    type Error = Error;
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        if value.kind != UpgradeClient::TYPE_STR {
            return Err(Error::WrongType {
                expected: UpgradeClient::TYPE_STR,
            });
        }

        let mut client_id = None;
        let mut client_type = None;
        let mut consensus_height = None;

        for attr in value.attributes {
            match attr.key.as_ref() {
                "client_id" => {
                    client_id = Some(ClientId(attr.value));
                }
                "client_type" => {
                    client_type = Some(ClientType(attr.value));
                }
                "consensus_height" => {
                    consensus_height =
                        Some(attr.value.parse().map_err(|e| Error::ParseHeight {
                            key: "consensus_height",
                            e,
                        })?);
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            client_id: client_id.ok_or(Error::MissingAttribute("client_id"))?,
            client_type: client_type.ok_or(Error::MissingAttribute("client_type"))?,
            consensus_height: consensus_height
                .ok_or(Error::MissingAttribute("consensus_height"))?,
        })
    }
}
