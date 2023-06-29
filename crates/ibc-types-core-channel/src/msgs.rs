//! Message definitions for all ICS4 domain types: channel open & close handshake datagrams, as well
//! as packets.

// Opening handshake messages.
mod chan_open_ack;
mod chan_open_confirm;
mod chan_open_init;
mod chan_open_try;

pub use chan_open_ack::MsgChannelOpenAck;
pub use chan_open_confirm::MsgChannelOpenConfirm;
pub use chan_open_init::MsgChannelOpenInit;
pub use chan_open_try::MsgChannelOpenTry;

// Closing handshake messages.
mod chan_close_confirm;
mod chan_close_init;

pub use chan_close_confirm::MsgChannelCloseConfirm;
pub use chan_close_init::MsgChannelCloseInit;

// Packet specific messages.
mod acknowledgement;
mod recv_packet;
mod timeout;
mod timeout_on_close;

pub use acknowledgement::MsgAcknowledgement;
pub use recv_packet::MsgRecvPacket;
pub use timeout::MsgTimeout;
pub use timeout_on_close::MsgTimeoutOnClose;

/// Enumeration of all possible messages that the ICS4 protocol processes.
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelMsg {
    OpenInit(MsgChannelOpenInit),
    OpenTry(MsgChannelOpenTry),
    OpenAck(MsgChannelOpenAck),
    OpenConfirm(MsgChannelOpenConfirm),
    CloseInit(MsgChannelCloseInit),
    CloseConfirm(MsgChannelCloseConfirm),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PacketMsg {
    Recv(MsgRecvPacket),
    Ack(MsgAcknowledgement),
    Timeout(MsgTimeout),
    TimeoutOnClose(MsgTimeoutOnClose),
}
