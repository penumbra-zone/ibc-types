use crate::prelude::*;

use ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;

use ibc_types_core_client::Height;
use ibc_types_domain_type::{DomainType, TypeUrl};

use crate::{packet::Sequence, Packet, PacketError};

impl TypeUrl for MsgTimeoutOnClose {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgTimeoutOnClose";
}

///
/// Message definition for packet timeout domain type.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgTimeoutOnClose {
    pub packet: Packet,
    pub next_seq_recv_on_b: Sequence,
    pub proof_unreceived_on_b: Vec<u8>,
    pub proof_close_on_b: Vec<u8>,
    pub proof_height_on_b: Height,
    pub signer: String,
}

impl DomainType for MsgTimeoutOnClose {
    type Proto = RawMsgTimeoutOnClose;
}

impl TryFrom<RawMsgTimeoutOnClose> for MsgTimeoutOnClose {
    type Error = PacketError;

    fn try_from(raw_msg: RawMsgTimeoutOnClose) -> Result<Self, Self::Error> {
        // TODO: Domain type verification for the next sequence: this should probably be > 0.
        Ok(MsgTimeoutOnClose {
            packet: raw_msg
                .packet
                .ok_or(PacketError::MissingPacket)?
                .try_into()?,
            next_seq_recv_on_b: Sequence::from(raw_msg.next_sequence_recv),
            proof_unreceived_on_b: raw_msg.proof_unreceived,
            proof_close_on_b: raw_msg.proof_close,
            proof_height_on_b: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(PacketError::MissingHeight)?,
            signer: raw_msg.signer,
        })
    }
}

impl From<MsgTimeoutOnClose> for RawMsgTimeoutOnClose {
    fn from(domain_msg: MsgTimeoutOnClose) -> Self {
        RawMsgTimeoutOnClose {
            packet: Some(domain_msg.packet.into()),
            proof_unreceived: domain_msg.proof_unreceived_on_b.into(),
            proof_close: domain_msg.proof_close_on_b.into(),
            proof_height: Some(domain_msg.proof_height_on_b.into()),
            next_sequence_recv: domain_msg.next_seq_recv_on_b.into(),
            signer: domain_msg.signer,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;
    use test_log::test;

    use super::test_util::*;
    use super::*;

    #[test]
    fn msg_timeout_on_close_try_from_raw() {
        let height = 50;
        let timeout_timestamp = 5;
        let raw = get_dummy_raw_msg_timeout_on_close(height, timeout_timestamp);

        let msg = MsgTimeoutOnClose::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgTimeoutOnClose::from(msg);
        assert_eq!(raw, raw_back);
    }

    #[test]
    fn parse_timeout_on_close_msg() {
        struct Test {
            name: String,
            raw: RawMsgTimeoutOnClose,
            want_pass: bool,
        }

        let height = 50;
        let timeout_timestamp = 5;
        let default_raw_msg = get_dummy_raw_msg_timeout_on_close(height, timeout_timestamp);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing packet".to_string(),
                raw: RawMsgTimeoutOnClose {
                    packet: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof of unreceived packet".to_string(),
                raw: RawMsgTimeoutOnClose {
                    proof_unreceived: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof of channel".to_string(),
                raw: RawMsgTimeoutOnClose {
                    proof_close: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgTimeoutOnClose {
                    proof_height: None,
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ];

        for test in tests {
            let res_msg = MsgTimeoutOnClose::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                res_msg.is_ok(),
                "MsgTimeoutOnClose::try_from raw failed for test {}, \nraw msg {:?} with error {:?}",
                test.name,
                test.raw,
                res_msg.err(),
            );
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;

    use crate::mocks::{get_dummy_bech32_account, get_dummy_proof};
    use crate::packet::test_utils::get_dummy_raw_packet;

    /// Returns a dummy `RawMsgTimeoutOnClose`, for testing only!
    /// The `height` parametrizes both the proof height as well as the timeout height.
    pub fn get_dummy_raw_msg_timeout_on_close(
        height: u64,
        timeout_timestamp: u64,
    ) -> RawMsgTimeoutOnClose {
        RawMsgTimeoutOnClose {
            packet: Some(get_dummy_raw_packet(height, timeout_timestamp)),
            proof_unreceived: get_dummy_proof(),
            proof_close: get_dummy_proof(),
            proof_height: Some(RawHeight {
                revision_number: 0,
                revision_height: height,
            }),
            next_sequence_recv: 1,
            signer: get_dummy_bech32_account(),
        }
    }
}
