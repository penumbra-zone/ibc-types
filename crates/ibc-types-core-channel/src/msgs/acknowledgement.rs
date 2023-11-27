use crate::prelude::*;

use ibc_proto::ibc::core::channel::v1::MsgAcknowledgement as RawMsgAcknowledgement;

use ibc_types_core_client::Height;
use ibc_types_core_commitment::MerkleProof;
use ibc_types_domain_type::DomainType;

use crate::{Packet, PacketError};

/*
use derive_more::Into;

/// A generic Acknowledgement type that modules may interpret as they like.
/// An acknowledgement cannot be empty.
#[derive(Clone, Debug, PartialEq, Eq, Into)]
pub struct Acknowledgement(pub Vec<u8>);

impl Acknowledgement {
    // Returns the data as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl AsRef<[u8]> for Acknowledgement {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl TryFrom<Vec<u8>> for Acknowledgement {
    type Error = PacketError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            Err(PacketError::InvalidAcknowledgement)
        } else {
            Ok(Self(bytes))
        }
    }
}
 */

///
/// Message definition for packet acknowledgements.
///
#[derive(Clone, Debug, PartialEq)]
pub struct MsgAcknowledgement {
    pub packet: Packet,
    pub acknowledgement: Vec<u8>,
    /// Proof of packet acknowledgement on the receiving chain
    pub proof_acked_on_b: MerkleProof,
    /// Height at which the commitment proof in this message were taken
    pub proof_height_on_b: Height,
    pub signer: String,
}

impl DomainType for MsgAcknowledgement {
    type Proto = RawMsgAcknowledgement;
}

impl TryFrom<RawMsgAcknowledgement> for MsgAcknowledgement {
    type Error = PacketError;

    fn try_from(raw_msg: RawMsgAcknowledgement) -> Result<Self, Self::Error> {
        if raw_msg.proof_acked.is_empty() {
            return Err(PacketError::InvalidAcknowledgement);
        }
        Ok(MsgAcknowledgement {
            packet: raw_msg
                .packet
                .ok_or(PacketError::MissingPacket)?
                .try_into()?,
            acknowledgement: raw_msg.acknowledgement,
            proof_acked_on_b: MerkleProof::decode(raw_msg.proof_acked.as_ref())
                .map_err(|_| PacketError::InvalidProof)?,
            proof_height_on_b: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(PacketError::MissingHeight)?,
            signer: raw_msg.signer,
        })
    }
}

impl From<MsgAcknowledgement> for RawMsgAcknowledgement {
    fn from(domain_msg: MsgAcknowledgement) -> Self {
        RawMsgAcknowledgement {
            packet: Some(domain_msg.packet.into()),
            acknowledgement: domain_msg.acknowledgement,
            signer: domain_msg.signer,
            proof_height: Some(domain_msg.proof_height_on_b.into()),
            proof_acked: domain_msg.proof_acked_on_b.encode_to_vec(),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use ibc_proto::ibc::core::channel::v1::MsgAcknowledgement as RawMsgAcknowledgement;
    use ibc_proto::ibc::core::channel::v1::Packet as RawPacket;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;

    use crate::{
        mocks::{get_dummy_bech32_account, get_dummy_proof},
        packet::test_utils::get_dummy_raw_packet,
    };

    /// Returns a dummy `RawMsgAcknowledgement`, for testing only!
    /// The `height` parametrizes both the proof height as well as the timeout height.
    pub fn get_dummy_raw_msg_acknowledgement(height: u64) -> RawMsgAcknowledgement {
        get_dummy_raw_msg_ack_with_packet(get_dummy_raw_packet(height, 1), height)
    }

    pub fn get_dummy_raw_msg_ack_with_packet(
        packet: RawPacket,
        height: u64,
    ) -> RawMsgAcknowledgement {
        RawMsgAcknowledgement {
            packet: Some(packet),
            acknowledgement: get_dummy_proof(),
            proof_acked: get_dummy_proof(),
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
    use super::test_util::*;
    use super::*;

    use test_log::test;

    use ibc_proto::ibc::core::channel::v1::MsgAcknowledgement as RawMsgAcknowledgement;

    use crate::mocks::get_dummy_bech32_account;
    use crate::PacketError;

    #[test]
    fn msg_acknowledgment_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgAcknowledgement,
            want_pass: bool,
        }

        let height = 50;
        let default_raw_msg = get_dummy_raw_msg_acknowledgement(height);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing packet".to_string(),
                raw: RawMsgAcknowledgement {
                    packet: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgAcknowledgement {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Empty signer".to_string(),
                raw: RawMsgAcknowledgement {
                    signer: get_dummy_bech32_account(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Empty proof acked".to_string(),
                raw: RawMsgAcknowledgement {
                    proof_acked: Vec::new(),
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ];

        for test in tests {
            let res_msg: Result<MsgAcknowledgement, PacketError> = test.raw.clone().try_into();

            assert_eq!(
                res_msg.is_ok(),
                test.want_pass,
                "MsgAcknowledgement::try_from failed for test {} \nraw message: {:?} with error: {:?}",
                test.name,
                test.raw,
                res_msg.err()
            );
        }
    }
}
