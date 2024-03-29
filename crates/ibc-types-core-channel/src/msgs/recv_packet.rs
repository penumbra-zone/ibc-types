use crate::prelude::*;

use ibc_types_core_client::Height;
use ibc_types_core_commitment::MerkleProof;
use ibc_types_domain_type::DomainType;

use crate::{Packet, PacketError};

use ibc_proto::ibc::core::channel::v1::MsgRecvPacket as RawMsgRecvPacket;

///
/// Message definition for the "packet receiving" datagram.
///
#[derive(Clone, Debug, PartialEq)]
pub struct MsgRecvPacket {
    /// The packet to be received
    pub packet: Packet,
    /// Proof of packet commitment on the sending chain
    pub proof_commitment_on_a: MerkleProof,
    /// Height at which the commitment proof in this message were taken
    pub proof_height_on_a: Height,
    /// The signer of the message
    pub signer: String,
}

impl DomainType for MsgRecvPacket {
    type Proto = RawMsgRecvPacket;
}

impl TryFrom<RawMsgRecvPacket> for MsgRecvPacket {
    type Error = PacketError;

    fn try_from(raw_msg: RawMsgRecvPacket) -> Result<Self, Self::Error> {
        if raw_msg.proof_commitment.is_empty() {
            return Err(PacketError::InvalidProof);
        }
        Ok(MsgRecvPacket {
            packet: raw_msg
                .packet
                .ok_or(PacketError::MissingPacket)?
                .try_into()?,
            proof_commitment_on_a: MerkleProof::decode(raw_msg.proof_commitment.as_ref())
                .map_err(|_| PacketError::InvalidProof)?,
            proof_height_on_a: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(PacketError::MissingHeight)?,
            signer: raw_msg.signer,
        })
    }
}

impl From<MsgRecvPacket> for RawMsgRecvPacket {
    fn from(domain_msg: MsgRecvPacket) -> Self {
        RawMsgRecvPacket {
            packet: Some(domain_msg.packet.into()),
            proof_commitment: domain_msg.proof_commitment_on_a.encode_to_vec(),
            proof_height: Some(domain_msg.proof_height_on_a.into()),
            signer: domain_msg.signer,
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use ibc_proto::ibc::core::channel::v1::MsgRecvPacket as RawMsgRecvPacket;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;

    use crate::mocks::{get_dummy_bech32_account, get_dummy_proof};
    use crate::packet::test_utils::get_dummy_raw_packet;
    use core::ops::Add;
    use core::time::Duration;
    use ibc_types_timestamp::Timestamp;

    /// Returns a dummy `RawMsgRecvPacket`, for testing only! The `height` parametrizes both the
    /// proof height as well as the timeout height.
    pub fn get_dummy_raw_msg_recv_packet(height: u64) -> RawMsgRecvPacket {
        let timestamp = Timestamp::now().add(Duration::from_secs(9));
        RawMsgRecvPacket {
            packet: Some(get_dummy_raw_packet(
                height,
                timestamp.unwrap().nanoseconds(),
            )),
            proof_commitment: get_dummy_proof(),
            proof_height: Some(RawHeight {
                revision_number: 0,
                revision_height: height,
            }),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::channel::v1::MsgRecvPacket as RawMsgRecvPacket;

    use super::test_util::*;
    use super::*;

    use crate::mocks::get_dummy_bech32_account;

    #[test]
    fn msg_recv_packet_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgRecvPacket,
            want_pass: bool,
        }

        let height = 20;
        let default_raw_msg = get_dummy_raw_msg_recv_packet(height);
        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing proof".to_string(),
                raw: RawMsgRecvPacket {
                    proof_commitment: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgRecvPacket {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Empty signer".to_string(),
                raw: RawMsgRecvPacket {
                    signer: get_dummy_bech32_account(),
                    ..default_raw_msg
                },
                want_pass: true,
            },
        ];

        for test in tests {
            let res_msg: Result<MsgRecvPacket, PacketError> = test.raw.clone().try_into();

            assert_eq!(
                res_msg.is_ok(),
                test.want_pass,
                "MsgRecvPacket::try_from failed for test {} \nraw message: {:?} with error: {:?}",
                test.name,
                test.raw,
                res_msg.err()
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_recv_packet(15);
        let msg = MsgRecvPacket::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgRecvPacket::from(msg.clone());
        let msg_back = MsgRecvPacket::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
