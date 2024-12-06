use crate::{packet, prelude::*, Packet, TimeoutHeight};
use alloc::borrow::ToOwned;

use ibc_types_core_connection::ConnectionId;
use ibc_types_timestamp::Timestamp;
use subtle_encoding::hex;
use tendermint::abci::{Event, TypedEvent};

use crate::{channel::Order, ChannelId, PortId};

use super::Error;

// TODO: consider deduplicating parser code using something like the internal
// Attributes structure in the connection impl.  For now, these implementations
// are almost -- but not entirely -- identical.

/// A `ChannelClose` event is emitted when a channel is closed as a result of a packet timing out. Note that
/// since optimistic packet sends (i.e. send a packet before channel handshake is complete) are supported,
/// we might not have a counterparty channel id value yet. This would happen if a packet is sent right
/// after a `ChannelOpenInit` message.
///
/// TODO: is this a "channel" event or a "packet" event?
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelClose {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: Option<ChannelId>,
    pub connection_id: ConnectionId,
    pub channel_ordering: Order,
}

impl ChannelClose {
    pub const TYPE_STR: &'static str = "channel_close";
}

impl TypedEvent for ChannelClose {}

impl From<ChannelClose> for Event {
    fn from(event: ChannelClose) -> Self {
        Event::new(
            ChannelClose::TYPE_STR,
            [
                ("port_id", event.port_id.0),
                ("channel_id", event.channel_id.0),
                ("counterparty_port_id", event.counterparty_port_id.0),
                (
                    "counterparty_channel_id",
                    event
                        .counterparty_channel_id
                        .map(|id| id.0)
                        .unwrap_or_default(),
                ),
                ("connection_id", event.connection_id.0),
                (
                    "packet_channel_ordering",
                    event.channel_ordering.as_str().to_owned(),
                ),
            ],
        )
    }
}

impl TryFrom<Event> for ChannelClose {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != ChannelClose::TYPE_STR {
            return Err(Error::WrongType {
                expected: ChannelClose::TYPE_STR,
            });
        }

        let mut port_id = None;
        let mut channel_id = None;
        let mut counterparty_port_id = None;
        let mut counterparty_channel_id = None;
        let mut connection_id = None;
        let mut channel_ordering = None;

        for attr in event.attributes {
            match attr.key_bytes() {
                b"port_id" => {
                    port_id = Some(PortId(String::from_utf8_lossy(attr.value_bytes()).into()));
                }
                b"channel_id" => {
                    channel_id = Some(ChannelId(
                        String::from_utf8_lossy(attr.value_bytes()).into(),
                    ));
                }
                b"counterparty_port_id" => {
                    counterparty_port_id =
                        Some(PortId(String::from_utf8_lossy(attr.value_bytes()).into()));
                }
                b"counterparty_channel_id" => {
                    counterparty_channel_id = if !attr.value_bytes().is_empty() {
                        Some(ChannelId(
                            String::from_utf8_lossy(attr.value_bytes()).into(),
                        ))
                    } else {
                        None
                    };
                }
                b"connection_id" => {
                    connection_id = Some(ConnectionId(
                        String::from_utf8_lossy(attr.value_bytes()).into(),
                    ));
                }
                b"packet_channel_ordering" => {
                    channel_ordering = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelOrder {
                                key: "packet_channel_ordering",
                                e,
                            })?,
                    )
                }
                unknown => {
                    return Err(Error::UnexpectedAttribute(
                        String::from_utf8_lossy(unknown).into(),
                    ))
                }
            }
        }

        Ok(Self {
            port_id: port_id.ok_or(Error::MissingAttribute("port_id"))?,
            channel_id: channel_id.ok_or(Error::MissingAttribute("channel_id"))?,
            counterparty_port_id: counterparty_port_id
                .ok_or(Error::MissingAttribute("counterparty_port_id"))?,
            counterparty_channel_id,
            connection_id: connection_id.ok_or(Error::MissingAttribute("connection_id"))?,
            channel_ordering: channel_ordering
                .ok_or(Error::MissingAttribute("packet_channel_ordering"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendPacket {
    pub packet_data: Vec<u8>,
    pub timeout_height: TimeoutHeight,
    pub timeout_timestamp: Timestamp,
    pub sequence: packet::Sequence,
    pub src_port_id: PortId,
    pub src_channel_id: ChannelId,
    pub dst_port_id: PortId,
    pub dst_channel_id: ChannelId,
    pub channel_ordering: Order,
    pub src_connection_id: ConnectionId,
}

impl SendPacket {
    pub const TYPE_STR: &'static str = "send_packet";

    pub fn new(packet: Packet, channel_ordering: Order, src_connection_id: ConnectionId) -> Self {
        Self {
            packet_data: packet.data,
            timeout_height: packet.timeout_height_on_b,
            timeout_timestamp: packet.timeout_timestamp_on_b,
            sequence: packet.sequence,
            src_port_id: packet.port_on_a,
            src_channel_id: packet.chan_on_a,
            dst_port_id: packet.port_on_b,
            dst_channel_id: packet.chan_on_b,
            channel_ordering,
            src_connection_id,
        }
    }
}

impl TypedEvent for SendPacket {}

impl From<SendPacket> for Event {
    fn from(event: SendPacket) -> Self {
        let mut attrs = Vec::with_capacity(11);
        attrs.push((
            "packet_data_hex",
            String::from_utf8(hex::encode(&event.packet_data)).unwrap(),
        ));
        // Conditionally include packet_data only if UTF-8 encodable
        // TODO: what's the right behavior here?
        // original impl just errors out entirely, doesn't seem right
        if let Ok(utf8_packet_data) = String::from_utf8(event.packet_data) {
            attrs.push(("packet_data", utf8_packet_data));
        }
        attrs.push(("packet_timeout_height", event.timeout_height.to_string()));
        attrs.push((
            "packet_timeout_timestamp",
            event.timeout_timestamp.nanoseconds().to_string(),
        ));
        attrs.push(("packet_sequence", event.sequence.to_string()));
        attrs.push(("packet_src_port", event.src_port_id.0));
        attrs.push(("packet_src_channel", event.src_channel_id.0));
        attrs.push(("packet_dst_port", event.dst_port_id.0));
        attrs.push(("packet_dst_channel", event.dst_channel_id.0));
        attrs.push((
            "packet_channel_ordering",
            event.channel_ordering.as_str().to_owned(),
        ));
        attrs.push(("packet_connection", event.src_connection_id.0));

        Event::new(SendPacket::TYPE_STR, attrs)
    }
}

impl TryFrom<Event> for SendPacket {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != SendPacket::TYPE_STR {
            return Err(Error::WrongType {
                expected: SendPacket::TYPE_STR,
            });
        }

        let mut packet_data = None;
        let mut timeout_height = None;
        let mut timeout_timestamp = None;
        let mut sequence = None;
        let mut src_port_id = None;
        let mut src_channel_id = None;
        let mut dst_port_id = None;
        let mut dst_channel_id = None;
        let mut channel_ordering = None;
        let mut src_connection_id = None;

        for attr in event.attributes {
            match attr.key_bytes() {
                b"packet_data" => {
                    let new_packet_data = attr.value_bytes();
                    if let Some(existing_packet_data) = packet_data {
                        if new_packet_data != existing_packet_data {
                            return Err(Error::MismatchedPacketData);
                        } else {
                            packet_data = Some(new_packet_data.into());
                        }
                    }
                }
                b"packet_data_hex" => {
                    let new_packet_data =
                        hex::decode(attr.value_bytes()).map_err(|e| Error::ParseHex {
                            key: "packet_data_hex",
                            e,
                        })?;
                    if let Some(existing_packet_data) = packet_data {
                        if new_packet_data != existing_packet_data {
                            return Err(Error::MismatchedPacketData);
                        } else {
                            packet_data = Some(new_packet_data);
                        }
                    }
                }
                b"packet_timeout_height" => {
                    timeout_height = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseTimeoutHeight {
                                key: "packet_timeout_height",
                                e,
                            })?,
                    );
                }
                b"packet_timeout_timestamp" => {
                    timeout_timestamp = Some(
                        Timestamp::from_nanoseconds(
                            String::from_utf8_lossy(attr.value_bytes())
                                .parse::<u64>()
                                .map_err(|e| Error::ParseTimeoutTimestampValue {
                                    key: "packet_timeout_timestamp",
                                    e,
                                })?,
                        )
                        .map_err(|e| Error::ParseTimeoutTimestamp {
                            key: "packet_timeout_timestamp",
                            e,
                        })?,
                    );
                }
                b"packet_sequence" => {
                    sequence = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseSequence {
                                key: "packet_sequence",
                                e,
                            })?,
                    );
                }
                b"packet_src_port" => {
                    src_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_src_port",
                                e,
                            })?,
                    );
                }
                b"packet_src_channel" => {
                    src_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_src_channel",
                                e,
                            })?,
                    );
                }
                b"packet_dst_port" => {
                    dst_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_dst_port",
                                e,
                            })?,
                    );
                }
                b"packet_dst_channel" => {
                    dst_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_dst_channel",
                                e,
                            })?,
                    );
                }
                b"packet_channel_ordering" => {
                    channel_ordering = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelOrder {
                                key: "packet_channel_ordering",
                                e,
                            })?,
                    );
                }
                b"packet_connection" => {
                    src_connection_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseConnectionId {
                                key: "packet_connection",
                                e,
                            })?,
                    );
                }
                unknown => {
                    return Err(Error::UnexpectedAttribute(
                        String::from_utf8_lossy(unknown).into(),
                    ))
                }
            }
        }

        Ok(Self {
            packet_data: packet_data
                .ok_or(Error::MissingAttribute("packet_data/packet_data_hex"))?,
            timeout_height: timeout_height
                .ok_or(Error::MissingAttribute("packet_timeout_height"))?,
            timeout_timestamp: timeout_timestamp
                .ok_or(Error::MissingAttribute("packet_timeout_timestamp"))?,
            sequence: sequence.ok_or(Error::MissingAttribute("packet_sequence"))?,
            src_port_id: src_port_id.ok_or(Error::MissingAttribute("packet_src_port"))?,
            dst_port_id: dst_port_id.ok_or(Error::MissingAttribute("packet_dst_port"))?,
            src_channel_id: src_channel_id.ok_or(Error::MissingAttribute("packet_src_channel"))?,
            dst_channel_id: dst_channel_id.ok_or(Error::MissingAttribute("packet_dst_channel"))?,
            channel_ordering: channel_ordering
                .ok_or(Error::MissingAttribute("packet_channel_ordering"))?,
            src_connection_id: src_connection_id
                .ok_or(Error::MissingAttribute("packet_connection"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceivePacket {
    pub packet_data: Vec<u8>,
    pub timeout_height: TimeoutHeight,
    pub timeout_timestamp: Timestamp,
    pub sequence: packet::Sequence,
    pub src_port_id: PortId,
    pub src_channel_id: ChannelId,
    pub dst_port_id: PortId,
    pub dst_channel_id: ChannelId,
    pub channel_ordering: Order,
    pub dst_connection_id: ConnectionId,
}

impl ReceivePacket {
    pub const TYPE_STR: &'static str = "recv_packet";

    pub fn new(packet: Packet, channel_ordering: Order, dst_connection_id: ConnectionId) -> Self {
        Self {
            packet_data: packet.data,
            timeout_height: packet.timeout_height_on_b,
            timeout_timestamp: packet.timeout_timestamp_on_b,
            sequence: packet.sequence,
            src_port_id: packet.port_on_a,
            src_channel_id: packet.chan_on_a,
            dst_port_id: packet.port_on_b,
            dst_channel_id: packet.chan_on_b,
            channel_ordering,
            dst_connection_id,
        }
    }
}

impl TypedEvent for ReceivePacket {}

impl From<ReceivePacket> for Event {
    fn from(event: ReceivePacket) -> Self {
        let mut attrs = Vec::with_capacity(11);
        attrs.push((
            "packet_data_hex",
            String::from_utf8(hex::encode(&event.packet_data)).unwrap(),
        ));
        // Conditionally include packet_data only if UTF-8 encodable
        // TODO: what's the right behavior here?
        // original impl just errors out entirely, doesn't seem right
        if let Ok(utf8_packet_data) = String::from_utf8(event.packet_data) {
            attrs.push(("packet_data", utf8_packet_data));
        }
        attrs.push(("packet_timeout_height", event.timeout_height.to_string()));
        attrs.push((
            "packet_timeout_timestamp",
            event.timeout_timestamp.nanoseconds().to_string(),
        ));
        attrs.push(("packet_sequence", event.sequence.to_string()));
        attrs.push(("packet_src_port", event.src_port_id.0));
        attrs.push(("packet_src_channel", event.src_channel_id.0));
        attrs.push(("packet_dst_port", event.dst_port_id.0));
        attrs.push(("packet_dst_channel", event.dst_channel_id.0));
        attrs.push((
            "packet_channel_ordering",
            event.channel_ordering.as_str().to_owned(),
        ));
        attrs.push(("packet_connection", event.dst_connection_id.0));

        Event::new(ReceivePacket::TYPE_STR, attrs)
    }
}

impl TryFrom<Event> for ReceivePacket {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != ReceivePacket::TYPE_STR {
            return Err(Error::WrongType {
                expected: ReceivePacket::TYPE_STR,
            });
        }

        let mut packet_data = None;
        let mut timeout_height = None;
        let mut timeout_timestamp = None;
        let mut sequence = None;
        let mut src_port_id = None;
        let mut src_channel_id = None;
        let mut dst_port_id = None;
        let mut dst_channel_id = None;
        let mut channel_ordering = None;
        let mut dst_connection_id = None;

        for attr in event.attributes {
            match attr.key_bytes() {
                b"packet_data" => {
                    let new_packet_data = attr.value_bytes().into();
                    if let Some(existing_packet_data) = packet_data {
                        if new_packet_data != existing_packet_data {
                            return Err(Error::MismatchedPacketData);
                        } else {
                            packet_data = Some(new_packet_data);
                        }
                    }
                }
                b"packet_data_hex" => {
                    let new_packet_data =
                        hex::decode(attr.value_bytes()).map_err(|e| Error::ParseHex {
                            key: "packet_data_hex",
                            e,
                        })?;
                    if let Some(existing_packet_data) = packet_data {
                        if new_packet_data != existing_packet_data {
                            return Err(Error::MismatchedPacketData);
                        } else {
                            packet_data = Some(new_packet_data);
                        }
                    }
                }
                b"packet_timeout_height" => {
                    timeout_height = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseTimeoutHeight {
                                key: "packet_timeout_height",
                                e,
                            })?,
                    );
                }
                b"packet_timeout_timestamp" => {
                    timeout_timestamp = Some(
                        Timestamp::from_nanoseconds(
                            String::from_utf8_lossy(attr.value_bytes())
                                .parse::<u64>()
                                .map_err(|e| Error::ParseTimeoutTimestampValue {
                                    key: "packet_timeout_timestamp",
                                    e,
                                })?,
                        )
                        .map_err(|e| Error::ParseTimeoutTimestamp {
                            key: "packet_timeout_timestamp",
                            e,
                        })?,
                    );
                }
                b"packet_sequence" => {
                    sequence = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseSequence {
                                key: "packet_sequence",
                                e,
                            })?,
                    );
                }
                b"packet_src_port" => {
                    src_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_src_port",
                                e,
                            })?,
                    );
                }
                b"packet_src_channel" => {
                    src_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_src_channel",
                                e,
                            })?,
                    );
                }
                b"packet_dst_port" => {
                    dst_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_dst_port",
                                e,
                            })?,
                    );
                }
                b"packet_dst_channel" => {
                    dst_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_dst_channel",
                                e,
                            })?,
                    );
                }
                b"packet_channel_ordering" => {
                    channel_ordering = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelOrder {
                                key: "packet_channel_ordering",
                                e,
                            })?,
                    );
                }
                b"packet_connection" => {
                    dst_connection_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseConnectionId {
                                key: "packet_connection",
                                e,
                            })?,
                    );
                }
                unknown => {
                    return Err(Error::UnexpectedAttribute(
                        String::from_utf8_lossy(unknown).into(),
                    ))
                }
            }
        }

        Ok(Self {
            packet_data: packet_data
                .ok_or(Error::MissingAttribute("packet_data/packet_data_hex"))?,
            timeout_height: timeout_height
                .ok_or(Error::MissingAttribute("packet_timeout_height"))?,
            timeout_timestamp: timeout_timestamp
                .ok_or(Error::MissingAttribute("packet_timeout_timestamp"))?,
            sequence: sequence.ok_or(Error::MissingAttribute("packet_sequence"))?,
            src_port_id: src_port_id.ok_or(Error::MissingAttribute("packet_src_port"))?,
            dst_port_id: dst_port_id.ok_or(Error::MissingAttribute("packet_dst_port"))?,
            src_channel_id: src_channel_id.ok_or(Error::MissingAttribute("packet_src_channel"))?,
            dst_channel_id: dst_channel_id.ok_or(Error::MissingAttribute("packet_dst_channel"))?,
            channel_ordering: channel_ordering
                .ok_or(Error::MissingAttribute("packet_channel_ordering"))?,
            dst_connection_id: dst_connection_id
                .ok_or(Error::MissingAttribute("packet_connection"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteAcknowledgement {
    pub packet_data: Vec<u8>,
    pub timeout_height: TimeoutHeight,
    pub timeout_timestamp: Timestamp,
    pub sequence: packet::Sequence,
    pub src_port_id: PortId,
    pub src_channel_id: ChannelId,
    pub dst_port_id: PortId,
    pub dst_channel_id: ChannelId,
    pub acknowledgement: Vec<u8>,
    pub dst_connection_id: ConnectionId,
}

impl WriteAcknowledgement {
    pub const TYPE_STR: &'static str = "write_acknowledgement";

    pub fn new(packet: Packet, acknowledgement: Vec<u8>, dst_connection_id: ConnectionId) -> Self {
        Self {
            packet_data: packet.data,
            timeout_height: packet.timeout_height_on_b,
            timeout_timestamp: packet.timeout_timestamp_on_b,
            sequence: packet.sequence,
            src_port_id: packet.port_on_a,
            src_channel_id: packet.chan_on_a,
            dst_port_id: packet.port_on_b,
            dst_channel_id: packet.chan_on_b,
            acknowledgement,
            dst_connection_id,
        }
    }
}

impl TypedEvent for WriteAcknowledgement {}

impl From<WriteAcknowledgement> for Event {
    fn from(event: WriteAcknowledgement) -> Self {
        let mut attrs = Vec::with_capacity(13);
        attrs.push((
            "packet_data_hex",
            String::from_utf8(hex::encode(&event.packet_data)).unwrap(),
        ));
        // Conditionally include packet_data only if UTF-8 encodable
        // TODO: what's the right behavior here?
        // original impl just errors out entirely, doesn't seem right
        if let Ok(utf8_packet_data) = String::from_utf8(event.packet_data) {
            attrs.push(("packet_data", utf8_packet_data));
        }
        attrs.push(("packet_timeout_height", event.timeout_height.to_string()));
        attrs.push((
            "packet_timeout_timestamp",
            event.timeout_timestamp.nanoseconds().to_string(),
        ));
        attrs.push(("packet_sequence", event.sequence.to_string()));
        attrs.push(("packet_src_port", event.src_port_id.0));
        attrs.push(("packet_src_channel", event.src_channel_id.0));
        attrs.push(("packet_dst_port", event.dst_port_id.0));
        attrs.push(("packet_dst_channel", event.dst_channel_id.0));
        attrs.push((
            "packet_ack_hex",
            String::from_utf8(hex::encode(&event.acknowledgement)).unwrap(),
        ));
        // Like packet_data, conditionally include packet_ack only if UTF-8 encodable.
        if let Ok(utf8_ack_data) = String::from_utf8(event.acknowledgement) {
            attrs.push(("packet_ack", utf8_ack_data));
        }
        attrs.push(("packet_connection", event.dst_connection_id.0));

        Event::new(WriteAcknowledgement::TYPE_STR, attrs)
    }
}

impl TryFrom<Event> for WriteAcknowledgement {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != WriteAcknowledgement::TYPE_STR {
            return Err(Error::WrongType {
                expected: WriteAcknowledgement::TYPE_STR,
            });
        }

        let mut packet_data = None;
        let mut timeout_height = None;
        let mut timeout_timestamp = None;
        let mut sequence = None;
        let mut src_port_id = None;
        let mut src_channel_id = None;
        let mut dst_port_id = None;
        let mut dst_channel_id = None;
        let mut acknowledgement = None;
        let mut dst_connection_id = None;

        for attr in event.attributes {
            match attr.key_bytes() {
                b"packet_data" => {
                    let new_packet_data = attr.value_bytes().into();
                    if let Some(existing_packet_data) = packet_data {
                        if new_packet_data != existing_packet_data {
                            return Err(Error::MismatchedPacketData);
                        } else {
                            packet_data = Some(new_packet_data);
                        }
                    }
                }
                b"packet_data_hex" => {
                    let new_packet_data =
                        hex::decode(attr.value_bytes()).map_err(|e| Error::ParseHex {
                            key: "packet_data_hex",
                            e,
                        })?;
                    if let Some(existing_packet_data) = packet_data {
                        if new_packet_data != existing_packet_data {
                            return Err(Error::MismatchedPacketData);
                        } else {
                            packet_data = Some(new_packet_data);
                        }
                    }
                }
                b"packet_timeout_height" => {
                    timeout_height = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseTimeoutHeight {
                                key: "packet_timeout_height",
                                e,
                            })?,
                    );
                }
                b"packet_timeout_timestamp" => {
                    timeout_timestamp = Some(
                        Timestamp::from_nanoseconds(
                            String::from_utf8_lossy(attr.value_bytes())
                                .parse::<u64>()
                                .map_err(|e| Error::ParseTimeoutTimestampValue {
                                    key: "packet_timeout_timestamp",
                                    e,
                                })?,
                        )
                        .map_err(|e| Error::ParseTimeoutTimestamp {
                            key: "packet_timeout_timestamp",
                            e,
                        })?,
                    );
                }
                b"packet_sequence" => {
                    sequence = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseSequence {
                                key: "packet_sequence",
                                e,
                            })?,
                    );
                }
                b"packet_src_port" => {
                    src_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_src_port",
                                e,
                            })?,
                    );
                }
                b"packet_src_channel" => {
                    src_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_src_channel",
                                e,
                            })?,
                    );
                }
                b"packet_dst_port" => {
                    dst_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_dst_port",
                                e,
                            })?,
                    );
                }
                b"packet_dst_channel" => {
                    dst_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_dst_channel",
                                e,
                            })?,
                    );
                }
                b"packet_ack" => {
                    let new_ack = attr.value_bytes().into();
                    if let Some(existing_ack) = acknowledgement {
                        if new_ack != existing_ack {
                            return Err(Error::MismatchedAcks);
                        } else {
                            acknowledgement = Some(new_ack);
                        }
                    }
                }
                b"packet_ack_hex" => {
                    let new_ack = hex::decode(attr.value_bytes()).map_err(|e| Error::ParseHex {
                        key: "packet_ack_hex",
                        e,
                    })?;

                    if let Some(existing_ack) = acknowledgement {
                        if new_ack != existing_ack {
                            return Err(Error::MismatchedAcks);
                        } else {
                            acknowledgement = Some(new_ack);
                        }
                    }
                }
                b"packet_connection" => {
                    dst_connection_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseConnectionId {
                                key: "packet_connection",
                                e,
                            })?,
                    );
                }
                unknown => {
                    return Err(Error::UnexpectedAttribute(
                        String::from_utf8_lossy(unknown).into(),
                    ))
                }
            }
        }

        Ok(Self {
            packet_data: packet_data
                .ok_or(Error::MissingAttribute("packet_data/packet_data_hex"))?,
            timeout_height: timeout_height
                .ok_or(Error::MissingAttribute("packet_timeout_height"))?,
            timeout_timestamp: timeout_timestamp
                .ok_or(Error::MissingAttribute("packet_timeout_timestamp"))?,
            sequence: sequence.ok_or(Error::MissingAttribute("packet_sequence"))?,
            src_port_id: src_port_id.ok_or(Error::MissingAttribute("packet_src_port"))?,
            dst_port_id: dst_port_id.ok_or(Error::MissingAttribute("packet_dst_port"))?,
            src_channel_id: src_channel_id.ok_or(Error::MissingAttribute("packet_src_channel"))?,
            dst_channel_id: dst_channel_id.ok_or(Error::MissingAttribute("packet_dst_channel"))?,
            acknowledgement: acknowledgement
                .ok_or(Error::MissingAttribute("packet_ack_hex or packet_ack"))?,
            dst_connection_id: dst_connection_id
                .ok_or(Error::MissingAttribute("packet_connection"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcknowledgePacket {
    pub timeout_height: TimeoutHeight,
    pub timeout_timestamp: Timestamp,
    pub sequence: packet::Sequence,
    pub src_port_id: PortId,
    pub src_channel_id: ChannelId,
    pub dst_port_id: PortId,
    pub dst_channel_id: ChannelId,
    pub channel_ordering: Order,
    pub src_connection_id: ConnectionId,
}

impl AcknowledgePacket {
    pub const TYPE_STR: &'static str = "acknowledge_packet";

    pub fn new(packet: Packet, channel_ordering: Order, src_connection_id: ConnectionId) -> Self {
        Self {
            timeout_height: packet.timeout_height_on_b,
            timeout_timestamp: packet.timeout_timestamp_on_b,
            sequence: packet.sequence,
            src_port_id: packet.port_on_a,
            src_channel_id: packet.chan_on_a,
            dst_port_id: packet.port_on_b,
            dst_channel_id: packet.chan_on_b,
            channel_ordering,
            src_connection_id,
        }
    }
}

impl TypedEvent for AcknowledgePacket {}

impl From<AcknowledgePacket> for Event {
    fn from(event: AcknowledgePacket) -> Self {
        let mut attrs = Vec::with_capacity(11);
        attrs.push(("packet_timeout_height", event.timeout_height.to_string()));
        attrs.push((
            "packet_timeout_timestamp",
            event.timeout_timestamp.nanoseconds().to_string(),
        ));
        attrs.push(("packet_sequence", event.sequence.to_string()));
        attrs.push(("packet_src_port", event.src_port_id.0));
        attrs.push(("packet_src_channel", event.src_channel_id.0));
        attrs.push(("packet_dst_port", event.dst_port_id.0));
        attrs.push(("packet_dst_channel", event.dst_channel_id.0));
        attrs.push((
            "packet_channel_ordering",
            event.channel_ordering.as_str().to_owned(),
        ));
        attrs.push(("packet_connection", event.src_connection_id.0));

        Event::new(AcknowledgePacket::TYPE_STR, attrs)
    }
}

impl TryFrom<Event> for AcknowledgePacket {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != AcknowledgePacket::TYPE_STR {
            return Err(Error::WrongType {
                expected: AcknowledgePacket::TYPE_STR,
            });
        }

        let mut timeout_height = None;
        let mut timeout_timestamp = None;
        let mut sequence = None;
        let mut src_port_id = None;
        let mut src_channel_id = None;
        let mut dst_port_id = None;
        let mut dst_channel_id = None;
        let mut channel_ordering = None;
        let mut src_connection_id = None;

        for attr in event.attributes {
            match attr.key_bytes() {
                b"packet_timeout_height" => {
                    timeout_height = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseTimeoutHeight {
                                key: "packet_timeout_height",
                                e,
                            })?,
                    );
                }
                b"packet_timeout_timestamp" => {
                    timeout_timestamp = Some(
                        Timestamp::from_nanoseconds(
                            String::from_utf8_lossy(attr.value_bytes())
                                .parse::<u64>()
                                .map_err(|e| Error::ParseTimeoutTimestampValue {
                                    key: "packet_timeout_timestamp",
                                    e,
                                })?,
                        )
                        .map_err(|e| Error::ParseTimeoutTimestamp {
                            key: "packet_timeout_timestamp",
                            e,
                        })?,
                    );
                }
                b"packet_sequence" => {
                    sequence = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseSequence {
                                key: "packet_sequence",
                                e,
                            })?,
                    );
                }
                b"packet_src_port" => {
                    src_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_src_port",
                                e,
                            })?,
                    );
                }
                b"packet_src_channel" => {
                    src_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_src_channel",
                                e,
                            })?,
                    );
                }
                b"packet_dst_port" => {
                    dst_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_dst_port",
                                e,
                            })?,
                    );
                }
                b"packet_dst_channel" => {
                    dst_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_dst_channel",
                                e,
                            })?,
                    );
                }
                b"packet_channel_ordering" => {
                    channel_ordering = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelOrder {
                                key: "packet_channel_ordering",
                                e,
                            })?,
                    );
                }
                b"packet_connection" => {
                    src_connection_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseConnectionId {
                                key: "packet_connection",
                                e,
                            })?,
                    );
                }
                unknown => {
                    return Err(Error::UnexpectedAttribute(
                        String::from_utf8_lossy(unknown).into(),
                    ))
                }
            }
        }

        Ok(Self {
            timeout_height: timeout_height
                .ok_or(Error::MissingAttribute("packet_timeout_height"))?,
            timeout_timestamp: timeout_timestamp
                .ok_or(Error::MissingAttribute("packet_timeout_timestamp"))?,
            sequence: sequence.ok_or(Error::MissingAttribute("packet_sequence"))?,
            src_port_id: src_port_id.ok_or(Error::MissingAttribute("packet_src_port"))?,
            dst_port_id: dst_port_id.ok_or(Error::MissingAttribute("packet_dst_port"))?,
            src_channel_id: src_channel_id.ok_or(Error::MissingAttribute("packet_src_channel"))?,
            dst_channel_id: dst_channel_id.ok_or(Error::MissingAttribute("packet_dst_channel"))?,
            channel_ordering: channel_ordering
                .ok_or(Error::MissingAttribute("packet_channel_ordering"))?,
            src_connection_id: src_connection_id
                .ok_or(Error::MissingAttribute("packet_connection"))?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeoutPacket {
    pub timeout_height: TimeoutHeight,
    pub timeout_timestamp: Timestamp,
    pub sequence: packet::Sequence,
    pub src_port_id: PortId,
    pub src_channel_id: ChannelId,
    pub dst_port_id: PortId,
    pub dst_channel_id: ChannelId,
    pub channel_ordering: Order,
}

impl TimeoutPacket {
    pub const TYPE_STR: &'static str = "timeout_packet";

    pub fn new(packet: Packet, channel_ordering: Order) -> Self {
        Self {
            timeout_height: packet.timeout_height_on_b,
            timeout_timestamp: packet.timeout_timestamp_on_b,
            sequence: packet.sequence,
            src_port_id: packet.port_on_a,
            src_channel_id: packet.chan_on_a,
            dst_port_id: packet.port_on_b,
            dst_channel_id: packet.chan_on_b,
            channel_ordering,
        }
    }
}

impl TypedEvent for TimeoutPacket {}

impl From<TimeoutPacket> for Event {
    fn from(event: TimeoutPacket) -> Self {
        let mut attrs = Vec::with_capacity(11);
        attrs.push(("packet_timeout_height", event.timeout_height.to_string()));
        attrs.push((
            "packet_timeout_timestamp",
            event.timeout_timestamp.nanoseconds().to_string(),
        ));
        attrs.push(("packet_sequence", event.sequence.to_string()));
        attrs.push(("packet_src_port", event.src_port_id.0));
        attrs.push(("packet_src_channel", event.src_channel_id.0));
        attrs.push(("packet_dst_port", event.dst_port_id.0));
        attrs.push(("packet_dst_channel", event.dst_channel_id.0));
        attrs.push((
            "packet_channel_ordering",
            event.channel_ordering.as_str().to_owned(),
        ));

        Event::new(TimeoutPacket::TYPE_STR, attrs)
    }
}

impl TryFrom<Event> for TimeoutPacket {
    type Error = Error;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if event.kind != TimeoutPacket::TYPE_STR {
            return Err(Error::WrongType {
                expected: TimeoutPacket::TYPE_STR,
            });
        }

        let mut timeout_height = None;
        let mut timeout_timestamp = None;
        let mut sequence = None;
        let mut src_port_id = None;
        let mut src_channel_id = None;
        let mut dst_port_id = None;
        let mut dst_channel_id = None;
        let mut channel_ordering = None;

        for attr in event.attributes {
            match attr.key_bytes() {
                b"packet_timeout_height" => {
                    timeout_height = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseTimeoutHeight {
                                key: "packet_timeout_height",
                                e,
                            })?,
                    );
                }
                b"packet_timeout_timestamp" => {
                    timeout_timestamp = Some(
                        Timestamp::from_nanoseconds(
                            String::from_utf8_lossy(attr.value_bytes())
                                .parse::<u64>()
                                .map_err(|e| Error::ParseTimeoutTimestampValue {
                                    key: "packet_timeout_timestamp",
                                    e,
                                })?,
                        )
                        .map_err(|e| Error::ParseTimeoutTimestamp {
                            key: "packet_timeout_timestamp",
                            e,
                        })?,
                    );
                }
                b"packet_sequence" => {
                    sequence = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseSequence {
                                key: "packet_sequence",
                                e,
                            })?,
                    );
                }
                b"packet_src_port" => {
                    src_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_src_port",
                                e,
                            })?,
                    );
                }
                b"packet_src_channel" => {
                    src_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_src_channel",
                                e,
                            })?,
                    );
                }
                b"packet_dst_port" => {
                    dst_port_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParsePortId {
                                key: "packet_dst_port",
                                e,
                            })?,
                    );
                }
                b"packet_dst_channel" => {
                    dst_channel_id = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelId {
                                key: "packet_dst_channel",
                                e,
                            })?,
                    );
                }
                b"packet_channel_ordering" => {
                    channel_ordering = Some(
                        String::from_utf8_lossy(attr.value_bytes())
                            .parse()
                            .map_err(|e| Error::ParseChannelOrder {
                                key: "packet_channel_ordering",
                                e,
                            })?,
                    );
                }
                unknown => {
                    return Err(Error::UnexpectedAttribute(
                        String::from_utf8_lossy(unknown).into(),
                    ))
                }
            }
        }

        Ok(Self {
            timeout_height: timeout_height
                .ok_or(Error::MissingAttribute("packet_timeout_height"))?,
            timeout_timestamp: timeout_timestamp
                .ok_or(Error::MissingAttribute("packet_timeout_timestamp"))?,
            sequence: sequence.ok_or(Error::MissingAttribute("packet_sequence"))?,
            src_port_id: src_port_id.ok_or(Error::MissingAttribute("packet_src_port"))?,
            dst_port_id: dst_port_id.ok_or(Error::MissingAttribute("packet_dst_port"))?,
            src_channel_id: src_channel_id.ok_or(Error::MissingAttribute("packet_src_channel"))?,
            dst_channel_id: dst_channel_id.ok_or(Error::MissingAttribute("packet_dst_channel"))?,
            channel_ordering: channel_ordering
                .ok_or(Error::MissingAttribute("packet_channel_ordering"))?,
        })
    }
}
