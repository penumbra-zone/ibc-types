use alloc::borrow::ToOwned;
use ibc_types_core_connection::ConnectionId;
use tendermint::abci::{Event, TypedEvent};

use crate::{ChannelId, PortId, Version};

use super::Error;

// TODO: consider deduplicating parser code using something like the internal
// Attributes structure in the connection impl.  For now, these implementations
// are almost -- but not entirely -- identical.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenInit {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub connection_id: ConnectionId,
    pub version: Version,
}

impl OpenInit {
    pub const TYPE_STR: &'static str = "channel_open_init";
}

impl TypedEvent for OpenInit {}

impl From<OpenInit> for Event {
    fn from(event: OpenInit) -> Self {
        Event::new(
            OpenInit::TYPE_STR,
            [
                ("port_id", event.port_id.0),
                ("channel_id", event.channel_id.0),
                ("counterparty_port_id", event.counterparty_port_id.0),
                ("connection_id", event.connection_id.0),
                ("version", event.version.0),
            ],
        )
    }
}

impl TryFrom<Event> for OpenInit {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != OpenInit::TYPE_STR {
            return Err(Error::WrongType {
                expected: OpenInit::TYPE_STR,
            });
        }

        let mut port_id = None;
        let mut channel_id = None;
        let mut counterparty_port_id = None;
        let mut connection_id = None;
        let mut version = None;

        for attr in event.attributes {
            match attr.key.as_ref() {
                "port_id" => {
                    port_id = Some(PortId(attr.value));
                }
                "channel_id" => {
                    channel_id = Some(ChannelId(attr.value));
                }
                "counterparty_port_id" => {
                    counterparty_port_id = Some(PortId(attr.value));
                }
                "connection_id" => {
                    connection_id = Some(ConnectionId(attr.value));
                }
                "version" => {
                    version = Some(Version(attr.value));
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            port_id: port_id.ok_or(Error::MissingAttribute("port_id"))?,
            channel_id: channel_id.ok_or(Error::MissingAttribute("channel_id"))?,
            counterparty_port_id: counterparty_port_id
                .ok_or(Error::MissingAttribute("counterparty_port_id"))?,
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
            version: version.ok_or(Error::MissingAttribute("version"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTry {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
    pub version: Version,
}

impl OpenTry {
    pub const TYPE_STR: &'static str = "channel_open_try";
}

impl TypedEvent for OpenTry {}

impl From<OpenTry> for Event {
    fn from(event: OpenTry) -> Self {
        Event::new(
            OpenTry::TYPE_STR,
            [
                ("port_id", event.port_id.0),
                ("channel_id", event.channel_id.0),
                ("counterparty_port_id", event.counterparty_port_id.0),
                ("counterparty_channel_id", event.counterparty_channel_id.0),
                ("connection_id", event.connection_id.0),
                ("version", event.version.0),
            ],
        )
    }
}

impl TryFrom<Event> for OpenTry {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != OpenTry::TYPE_STR {
            return Err(Error::WrongType {
                expected: OpenTry::TYPE_STR,
            });
        }

        let mut port_id = None;
        let mut channel_id = None;
        let mut counterparty_port_id = None;
        let mut counterparty_channel_id = None;
        let mut connection_id = None;
        let mut version = None;

        for attr in event.attributes {
            match attr.key.as_ref() {
                "port_id" => {
                    port_id = Some(PortId(attr.value));
                }
                "channel_id" => {
                    channel_id = Some(ChannelId(attr.value));
                }
                "counterparty_port_id" => {
                    counterparty_port_id = Some(PortId(attr.value));
                }
                "counterparty_channel_id" => {
                    counterparty_channel_id = Some(ChannelId(attr.value));
                }
                "connection_id" => {
                    connection_id = Some(ConnectionId(attr.value));
                }
                "version" => {
                    version = Some(Version(attr.value));
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            port_id: port_id.ok_or(Error::MissingAttribute("port_id"))?,
            channel_id: channel_id.ok_or(Error::MissingAttribute("channel_id"))?,
            counterparty_port_id: counterparty_port_id
                .ok_or(Error::MissingAttribute("counterparty_port_id"))?,
            counterparty_channel_id: counterparty_channel_id
                .ok_or(Error::MissingAttribute("counterparty_channel_id"))?,
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
            version: version.ok_or(Error::MissingAttribute("version"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAck {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
}

impl OpenAck {
    pub const TYPE_STR: &'static str = "channel_open_ack";
}

impl TypedEvent for OpenAck {}

impl From<OpenAck> for Event {
    fn from(event: OpenAck) -> Self {
        Event::new(
            OpenAck::TYPE_STR,
            [
                ("port_id", event.port_id.0),
                ("channel_id", event.channel_id.0),
                ("counterparty_port_id", event.counterparty_port_id.0),
                ("counterparty_channel_id", event.counterparty_channel_id.0),
                ("connection_id", event.connection_id.0),
            ],
        )
    }
}

impl TryFrom<Event> for OpenAck {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != OpenAck::TYPE_STR {
            return Err(Error::WrongType {
                expected: OpenAck::TYPE_STR,
            });
        }

        let mut port_id = None;
        let mut channel_id = None;
        let mut counterparty_port_id = None;
        let mut counterparty_channel_id = None;
        let mut connection_id = None;

        for attr in event.attributes {
            match attr.key.as_ref() {
                "port_id" => {
                    port_id = Some(PortId(attr.value));
                }
                "channel_id" => {
                    channel_id = Some(ChannelId(attr.value));
                }
                "counterparty_port_id" => {
                    counterparty_port_id = Some(PortId(attr.value));
                }
                "counterparty_channel_id" => {
                    counterparty_channel_id = Some(ChannelId(attr.value));
                }
                "connection_id" => {
                    connection_id = Some(ConnectionId(attr.value));
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            port_id: port_id.ok_or(Error::MissingAttribute("port_id"))?,
            channel_id: channel_id.ok_or(Error::MissingAttribute("channel_id"))?,
            counterparty_port_id: counterparty_port_id
                .ok_or(Error::MissingAttribute("counterparty_port_id"))?,
            counterparty_channel_id: counterparty_channel_id
                .ok_or(Error::MissingAttribute("counterparty_channel_id"))?,
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenConfirm {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
}

impl OpenConfirm {
    pub const TYPE_STR: &'static str = "channel_open_confirm";
}

impl TypedEvent for OpenConfirm {}

impl From<OpenConfirm> for Event {
    fn from(event: OpenConfirm) -> Self {
        Event::new(
            OpenConfirm::TYPE_STR,
            [
                ("port_id", event.port_id.0),
                ("channel_id", event.channel_id.0),
                ("counterparty_port_id", event.counterparty_port_id.0),
                ("counterparty_channel_id", event.counterparty_channel_id.0),
                ("connection_id", event.connection_id.0),
            ],
        )
    }
}

impl TryFrom<Event> for OpenConfirm {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != OpenConfirm::TYPE_STR {
            return Err(Error::WrongType {
                expected: OpenConfirm::TYPE_STR,
            });
        }

        let mut port_id = None;
        let mut channel_id = None;
        let mut counterparty_port_id = None;
        let mut counterparty_channel_id = None;
        let mut connection_id = None;

        for attr in event.attributes {
            match attr.key.as_ref() {
                "port_id" => {
                    port_id = Some(PortId(attr.value));
                }
                "channel_id" => {
                    channel_id = Some(ChannelId(attr.value));
                }
                "counterparty_port_id" => {
                    counterparty_port_id = Some(PortId(attr.value));
                }
                "counterparty_channel_id" => {
                    counterparty_channel_id = Some(ChannelId(attr.value));
                }
                "connection_id" => {
                    connection_id = Some(ConnectionId(attr.value));
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            port_id: port_id.ok_or(Error::MissingAttribute("port_id"))?,
            channel_id: channel_id.ok_or(Error::MissingAttribute("channel_id"))?,
            counterparty_port_id: counterparty_port_id
                .ok_or(Error::MissingAttribute("counterparty_port_id"))?,
            counterparty_channel_id: counterparty_channel_id
                .ok_or(Error::MissingAttribute("counterparty_channel_id"))?,
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseInit {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
}

impl CloseInit {
    pub const TYPE_STR: &'static str = "channel_close_init";
}

impl TypedEvent for CloseInit {}

impl From<CloseInit> for Event {
    fn from(event: CloseInit) -> Self {
        Event::new(
            CloseInit::TYPE_STR,
            [
                ("port_id", event.port_id.0),
                ("channel_id", event.channel_id.0),
                ("counterparty_port_id", event.counterparty_port_id.0),
                ("counterparty_channel_id", event.counterparty_channel_id.0),
                ("connection_id", event.connection_id.0),
            ],
        )
    }
}

impl TryFrom<Event> for CloseInit {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != CloseInit::TYPE_STR {
            return Err(Error::WrongType {
                expected: CloseInit::TYPE_STR,
            });
        }

        let mut port_id = None;
        let mut channel_id = None;
        let mut counterparty_port_id = None;
        let mut counterparty_channel_id = None;
        let mut connection_id = None;

        for attr in event.attributes {
            match attr.key.as_ref() {
                "port_id" => {
                    port_id = Some(PortId(attr.value));
                }
                "channel_id" => {
                    channel_id = Some(ChannelId(attr.value));
                }
                "counterparty_port_id" => {
                    counterparty_port_id = Some(PortId(attr.value));
                }
                "counterparty_channel_id" => {
                    counterparty_channel_id = Some(ChannelId(attr.value));
                }
                "connection_id" => {
                    connection_id = Some(ConnectionId(attr.value));
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            port_id: port_id.ok_or(Error::MissingAttribute("port_id"))?,
            channel_id: channel_id.ok_or(Error::MissingAttribute("channel_id"))?,
            counterparty_port_id: counterparty_port_id
                .ok_or(Error::MissingAttribute("counterparty_port_id"))?,
            counterparty_channel_id: counterparty_channel_id
                .ok_or(Error::MissingAttribute("counterparty_channel_id"))?,
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseConfirm {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,
    pub connection_id: ConnectionId,
}

impl CloseConfirm {
    pub const TYPE_STR: &'static str = "channel_close_confirm";
}

impl TypedEvent for CloseConfirm {}

impl From<CloseConfirm> for Event {
    fn from(event: CloseConfirm) -> Self {
        Event::new(
            CloseConfirm::TYPE_STR,
            [
                ("port_id", event.port_id.0),
                ("channel_id", event.channel_id.0),
                ("counterparty_port_id", event.counterparty_port_id.0),
                ("counterparty_channel_id", event.counterparty_channel_id.0),
                ("connection_id", event.connection_id.0),
            ],
        )
    }
}

impl TryFrom<Event> for CloseConfirm {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != CloseConfirm::TYPE_STR {
            return Err(Error::WrongType {
                expected: CloseConfirm::TYPE_STR,
            });
        }

        let mut port_id = None;
        let mut channel_id = None;
        let mut counterparty_port_id = None;
        let mut counterparty_channel_id = None;
        let mut connection_id = None;

        for attr in event.attributes {
            match attr.key.as_ref() {
                "port_id" => {
                    port_id = Some(PortId(attr.value));
                }
                "channel_id" => {
                    channel_id = Some(ChannelId(attr.value));
                }
                "counterparty_port_id" => {
                    counterparty_port_id = Some(PortId(attr.value));
                }
                "counterparty_channel_id" => {
                    counterparty_channel_id = Some(ChannelId(attr.value));
                }
                "connection_id" => {
                    connection_id = Some(ConnectionId(attr.value));
                }
                unknown => return Err(Error::UnexpectedAttribute(unknown.to_owned())),
            }
        }

        Ok(Self {
            port_id: port_id.ok_or(Error::MissingAttribute("port_id"))?,
            channel_id: channel_id.ok_or(Error::MissingAttribute("channel_id"))?,
            counterparty_port_id: counterparty_port_id
                .ok_or(Error::MissingAttribute("counterparty_port_id"))?,
            counterparty_channel_id: counterparty_channel_id
                .ok_or(Error::MissingAttribute("counterparty_channel_id"))?,
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
        })
    }
}
