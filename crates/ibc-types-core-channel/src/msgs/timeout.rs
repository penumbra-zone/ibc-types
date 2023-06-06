use crate::prelude::*;

use ibc_proto::ibc::core::channel::v1::MsgTimeout as RawMsgTimeout;

use ibc_types_core_client::Height;
use ibc_types_domain_type::{DomainType, TypeUrl};

use crate::{packet::Sequence, Packet, PacketError};

impl TypeUrl for MsgTimeout {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgTimeout";
}

///
/// Message definition for packet timeout domain type,
/// which is sent on chain A and needs to prove that a previously sent packet was not received on chain B
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgTimeout {
    pub packet: Packet,
    pub next_seq_recv_on_b: Sequence,
    pub proof_unreceived_on_b: Vec<u8>,
    pub proof_height_on_b: Height,
    pub signer: String,
}

impl DomainType for MsgTimeout {
    type Proto = RawMsgTimeout;
}

impl TryFrom<RawMsgTimeout> for MsgTimeout {
    type Error = PacketError;

    fn try_from(raw_msg: RawMsgTimeout) -> Result<Self, Self::Error> {
        // TODO: Domain type verification for the next sequence: this should probably be > 0.
        if raw_msg.proof_unreceived.is_empty() {
            return Err(PacketError::InvalidProof);
        }
        Ok(MsgTimeout {
            packet: raw_msg
                .packet
                .ok_or(PacketError::MissingPacket)?
                .try_into()?,
            next_seq_recv_on_b: Sequence::from(raw_msg.next_sequence_recv),
            proof_unreceived_on_b: raw_msg.proof_unreceived,
            proof_height_on_b: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(PacketError::MissingHeight)?,
            signer: raw_msg.signer,
        })
    }
}

impl From<MsgTimeout> for RawMsgTimeout {
    fn from(domain_msg: MsgTimeout) -> Self {
        RawMsgTimeout {
            packet: Some(domain_msg.packet.into()),
            proof_unreceived: domain_msg.proof_unreceived_on_b.into(),
            proof_height: Some(domain_msg.proof_height_on_b.into()),
            next_sequence_recv: domain_msg.next_seq_recv_on_b.into(),
            signer: domain_msg.signer,
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use ibc_proto::ibc::core::channel::v1::MsgTimeout as RawMsgTimeout;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;

    use crate::mocks::{get_dummy_bech32_account, get_dummy_proof};
    use crate::packet::test_utils::get_dummy_raw_packet;

    /// Returns a dummy `RawMsgTimeout`, for testing only!
    /// The `height` parametrizes both the proof height as well as the timeout height.
    pub fn get_dummy_raw_msg_timeout(
        proof_height: u64,
        timeout_height: u64,
        timeout_timestamp: u64,
    ) -> RawMsgTimeout {
        RawMsgTimeout {
            packet: Some(get_dummy_raw_packet(timeout_height, timeout_timestamp)),
            proof_unreceived: get_dummy_proof(),
            proof_height: Some(RawHeight {
                revision_number: 0,
                revision_height: proof_height,
            }),
            next_sequence_recv: 1,
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod test {

    use test_log::test;

    use ibc_proto::ibc::core::channel::v1::MsgTimeout as RawMsgTimeout;

    use super::test_util::*;
    use super::*;
    use crate::mocks::get_dummy_bech32_account;

    #[test]
    fn msg_timeout_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgTimeout,
            want_pass: bool,
        }

        let proof_height = 50;
        let timeout_height = proof_height;
        let timeout_timestamp = 0;
        let default_raw_msg =
            get_dummy_raw_msg_timeout(proof_height, timeout_height, timeout_timestamp);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing packet".to_string(),
                raw: RawMsgTimeout {
                    packet: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof".to_string(),
                raw: RawMsgTimeout {
                    proof_unreceived: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgTimeout {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Empty signer".to_string(),
                raw: RawMsgTimeout {
                    signer: get_dummy_bech32_account(),
                    ..default_raw_msg
                },
                want_pass: true,
            },
        ];

        for test in tests {
            let res_msg: Result<MsgTimeout, PacketError> = test.raw.clone().try_into();

            assert_eq!(
                res_msg.is_ok(),
                test.want_pass,
                "MsgTimeout::try_from failed for test {} \nraw message: {:?} with error: {:?}",
                test.name,
                test.raw,
                res_msg.err()
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_timeout(15, 20, 0);
        let msg = MsgTimeout::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgTimeout::from(msg.clone());
        let msg_back = MsgTimeout::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
