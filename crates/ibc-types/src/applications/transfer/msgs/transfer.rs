//! This is the definition of a transfer messages that an application submits to a chain.

use crate::applications::transfer::packet::PacketData;
use crate::prelude::*;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::applications::transfer::v1::MsgTransfer as RawMsgTransfer;
use ibc_proto::protobuf::Protobuf;

use crate::applications::transfer::error::TokenTransferError;
use crate::core::ics04_channel::timeout::TimeoutHeight;
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::timestamp::Timestamp;
use crate::tx_msg::Msg;

pub const TYPE_URL: &str = "/ibc.applications.transfer.v1.MsgTransfer";

/// Message used to build an ICS20 token transfer packet.
///
/// Note that this message is not a packet yet, as it lacks the proper sequence
/// number, and destination port/channel. This is by design. The sender of the
/// packet, which might be the user of a command line application, should only
/// have to specify the information related to the transfer of the token, and
/// let the library figure out how to build the packet properly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgTransfer {
    /// the port on which the packet will be sent
    pub port_id_on_a: PortId,
    /// the channel by which the packet will be sent
    pub chan_id_on_a: ChannelId,
    /// token transfer packet data of the packet that will be sent
    pub packet_data: PacketData,
    /// Timeout height relative to the current block height.
    /// The timeout is disabled when set to None.
    pub timeout_height_on_b: TimeoutHeight,
    /// Timeout timestamp relative to the current block timestamp.
    /// The timeout is disabled when set to 0.
    pub timeout_timestamp_on_b: Timestamp,
}

impl Msg for MsgTransfer {
    type Raw = RawMsgTransfer;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl TryFrom<RawMsgTransfer> for MsgTransfer {
    type Error = TokenTransferError;

    fn try_from(raw_msg: RawMsgTransfer) -> Result<Self, Self::Error> {
        let timeout_timestamp_on_b = Timestamp::from_nanoseconds(raw_msg.timeout_timestamp)
            .map_err(|_| TokenTransferError::InvalidPacketTimeoutTimestamp {
                timestamp: raw_msg.timeout_timestamp,
            })?;

        let timeout_height_on_b: TimeoutHeight =
            raw_msg.timeout_height.try_into().map_err(|e| {
                TokenTransferError::InvalidPacketTimeoutHeight {
                    context: format!("invalid timeout height {e}"),
                }
            })?;

        Ok(MsgTransfer {
            port_id_on_a: raw_msg.source_port.parse().map_err(|e| {
                TokenTransferError::InvalidPortId {
                    context: raw_msg.source_port.clone(),
                    validation_error: e,
                }
            })?,
            chan_id_on_a: raw_msg.source_channel.parse().map_err(|e| {
                TokenTransferError::InvalidChannelId {
                    context: raw_msg.source_channel.clone(),
                    validation_error: e,
                }
            })?,
            packet_data: PacketData {
                token: raw_msg
                    .token
                    .ok_or(TokenTransferError::InvalidToken)?
                    .try_into()
                    .map_err(|_| TokenTransferError::InvalidToken)?,
                sender: raw_msg.sender.parse().map_err(TokenTransferError::Signer)?,
                receiver: raw_msg
                    .receiver
                    .parse()
                    .map_err(TokenTransferError::Signer)?,
                memo: raw_msg.memo.into(),
            },
            timeout_height_on_b,
            timeout_timestamp_on_b,
        })
    }
}

impl From<MsgTransfer> for RawMsgTransfer {
    fn from(domain_msg: MsgTransfer) -> Self {
        RawMsgTransfer {
            source_port: domain_msg.port_id_on_a.to_string(),
            source_channel: domain_msg.chan_id_on_a.to_string(),
            token: Some(domain_msg.packet_data.token.into()),
            sender: domain_msg.packet_data.sender.to_string(),
            receiver: domain_msg.packet_data.receiver.to_string(),
            timeout_height: domain_msg.timeout_height_on_b.into(),
            timeout_timestamp: domain_msg.timeout_timestamp_on_b.nanoseconds(),
            memo: domain_msg.packet_data.memo.to_string(),
        }
    }
}

impl Protobuf<RawMsgTransfer> for MsgTransfer {}

impl TryFrom<Any> for MsgTransfer {
    type Error = TokenTransferError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            TYPE_URL => {
                MsgTransfer::decode_vec(&raw.value).map_err(TokenTransferError::DecodeRawMsg)
            }
            _ => Err(TokenTransferError::UnknownMsgType {
                msg_type: raw.type_url,
            }),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use alloc::borrow::ToOwned;
    use core::ops::Add;
    use core::time::Duration;
    use primitive_types::U256;

    use super::MsgTransfer;
    use crate::applications::transfer::packet::PacketData;
    use crate::applications::transfer::Coin;
    use crate::core::ics04_channel::packet::{Packet, Sequence};
    use crate::core::ics04_channel::timeout::TimeoutHeight;
    use crate::signer::Signer;
    use crate::{
        applications::transfer::BaseCoin,
        core::ics24_host::identifier::{ChannelId, PortId},
        test_utils::get_dummy_bech32_account,
        timestamp::Timestamp,
    };

    // Returns a dummy ICS20 `MsgTransfer`. If no `timeout_timestamp` is
    // specified, a timestamp of 10 seconds in the future is used.
    pub fn get_dummy_msg_transfer(
        timeout_height: TimeoutHeight,
        timeout_timestamp: Option<Timestamp>,
    ) -> MsgTransfer {
        let address: Signer = get_dummy_bech32_account().as_str().parse().unwrap();
        MsgTransfer {
            port_id_on_a: PortId::default(),
            chan_id_on_a: ChannelId::default(),
            packet_data: PacketData {
                token: BaseCoin {
                    denom: "uatom".parse().unwrap(),
                    amount: U256::from(10).into(),
                }
                .into(),
                sender: address.clone(),
                receiver: address,
                memo: "".to_owned().into(),
            },
            timeout_timestamp_on_b: timeout_timestamp
                .unwrap_or_else(|| Timestamp::now().add(Duration::from_secs(10)).unwrap()),
            timeout_height_on_b: timeout_height,
        }
    }

    pub fn get_dummy_transfer_packet(msg: MsgTransfer, sequence: Sequence) -> Packet {
        let coin = Coin {
            denom: msg.packet_data.token.denom.clone(),
            amount: msg.packet_data.token.amount,
        };

        let data = {
            let data = PacketData {
                token: coin,
                sender: msg.packet_data.sender.clone(),
                receiver: msg.packet_data.receiver.clone(),
                memo: msg.packet_data.memo.clone(),
            };
            serde_json::to_vec(&data).expect("PacketData's infallible Serialize impl failed")
        };

        Packet {
            seq_on_a: sequence,
            port_id_on_a: msg.port_id_on_a,
            chan_id_on_a: msg.chan_id_on_a,
            port_id_on_b: PortId::default(),
            chan_id_on_b: ChannelId::default(),
            data,
            timeout_height_on_b: msg.timeout_height_on_b,
            timeout_timestamp_on_b: msg.timeout_timestamp_on_b,
        }
    }
}
