use prost::Message;

use crate::core::ics03_connection::delay::verify_conn_delay_passed;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::commitment::compute_packet_commitment;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
use crate::core::ics24_host::path::{
    ChannelEndPath, ClientConsensusStatePath, CommitmentPath, ReceiptPath, SeqRecvPath,
};
use crate::core::ics24_host::Path;
use crate::core::{ContextError, ValidationContext};
use crate::prelude::*;

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgTimeout) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let chan_end_on_a = ctx_a.channel_end(&ChannelEndPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
    ))?;

    if !chan_end_on_a.state_matches(&State::Open) {
        return Err(PacketError::ChannelClosed {
            channel_id: msg.packet.chan_id_on_a.clone(),
        }
        .into());
    }

    let counterparty = Counterparty::new(
        msg.packet.port_id_on_b.clone(),
        Some(msg.packet.chan_id_on_b.clone()),
    );

    if !chan_end_on_a.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: msg.packet.port_id_on_b.clone(),
            channel_id: msg.packet.chan_id_on_b.clone(),
        }
        .into());
    }

    let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();
    let conn_end_on_a = ctx_a.connection_end(&conn_id_on_a)?;

    //verify packet commitment
    let commitment_path_on_a = CommitmentPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
        msg.packet.seq_on_a,
    );
    let commitment_on_a = match ctx_a.get_packet_commitment(&commitment_path_on_a) {
        Ok(commitment_on_a) => commitment_on_a,

        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        Err(_) => return Ok(()),
    };

    let expected_commitment_on_a = compute_packet_commitment(
        &msg.packet.data,
        &msg.packet.timeout_height_on_b,
        &msg.packet.timeout_timestamp_on_b,
    );
    if commitment_on_a != expected_commitment_on_a {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: msg.packet.seq_on_a,
        }
        .into());
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;
        client_state_of_b_on_a.confirm_not_frozen()?;
        client_state_of_b_on_a.validate_proof_height(msg.proof_height_on_b)?;

        // check that timeout height or timeout timestamp has passed on the other end
        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(client_id_on_a, &msg.proof_height_on_b);
        let consensus_state_of_b_on_a = ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let timestamp_of_b = consensus_state_of_b_on_a.timestamp();

        if !msg.packet.timed_out(&timestamp_of_b, msg.proof_height_on_b) {
            return Err(PacketError::PacketTimeoutNotReached {
                timeout_height: msg.packet.timeout_height_on_b,
                chain_height: msg.proof_height_on_b,
                timeout_timestamp: msg.packet.timeout_timestamp_on_b,
                chain_timestamp: timestamp_of_b,
            }
            .into());
        }

        verify_conn_delay_passed(ctx_a, msg.proof_height_on_b, &conn_end_on_a)?;

        let next_seq_recv_verification_result = if chan_end_on_a.order_matches(&Order::Ordered) {
            if msg.packet.seq_on_a < msg.next_seq_recv_on_b {
                return Err(PacketError::InvalidPacketSequence {
                    given_sequence: msg.packet.seq_on_a,
                    next_sequence: msg.next_seq_recv_on_b,
                }
                .into());
            }
            let seq_recv_path_on_b =
                SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);

            let mut value = Vec::new();
            u64::from(msg.packet.seq_on_a)
                .encode(&mut value)
                .map_err(|_| PacketError::CannotEncodeSequence {
                    sequence: msg.packet.seq_on_a,
                })?;

            client_state_of_b_on_a.verify_membership(
                conn_end_on_a.counterparty().prefix(),
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                Path::SeqRecv(seq_recv_path_on_b),
                value,
            )
        } else {
            let receipt_path_on_b = ReceiptPath::new(
                &msg.packet.port_id_on_b,
                &msg.packet.chan_id_on_b,
                msg.packet.seq_on_a,
            );

            client_state_of_b_on_a.verify_non_membership(
                conn_end_on_a.counterparty().prefix(),
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                Path::Receipt(receipt_path_on_b),
            )
        };
        next_seq_recv_verification_result
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.next_seq_recv_on_b,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::commitment::compute_packet_commitment;
    use crate::core::ExecutionContext;
    use crate::prelude::*;
    use crate::timestamp::Timestamp;
    use rstest::*;

    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::commitment::PacketCommitment;
    use crate::core::ics04_channel::handler::timeout::validate;
    use crate::core::ics04_channel::msgs::timeout::test_util::get_dummy_raw_msg_timeout;
    use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;

    pub struct Fixture {
        pub context: MockContext,
        pub client_height: Height,
        pub msg: MsgTimeout,
        pub packet_commitment: PacketCommitment,
        pub conn_end_on_a: ConnectionEnd,
        pub chan_end_on_a_unordered: ChannelEnd,
        pub chan_end_on_a_ordered: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let context = MockContext::default();

        let client_height = Height::new(0, 2).unwrap();
        let msg_proof_height = 2;
        let msg_timeout_height = 5;
        let timeout_timestamp = Timestamp::now().nanoseconds();

        let msg = MsgTimeout::try_from(get_dummy_raw_msg_timeout(
            msg_proof_height,
            msg_timeout_height,
            timeout_timestamp,
        ))
        .unwrap();
        let packet = msg.packet.clone();

        let packet_commitment = compute_packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a_unordered = ChannelEnd::new(
            State::Open,
            Order::Unordered,
            Counterparty::new(packet.port_id_on_b.clone(), Some(packet.chan_id_on_b)),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        );

        let mut chan_end_on_a_ordered = chan_end_on_a_unordered.clone();
        chan_end_on_a_ordered.ordering = Order::Ordered;

        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Open,
            ClientId::default(),
            ConnectionCounterparty::new(
                ClientId::default(),
                Some(ConnectionId::default()),
                Default::default(),
            ),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        Fixture {
            context,
            client_height,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a_unordered,
            chan_end_on_a_ordered,
        }
    }

    #[rstest]
    fn timeout_fail_no_channel(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_height,
            ..
        } = fixture;
        let context = context.with_client(&ClientId::default(), client_height);
        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    #[rstest]
    fn timeout_fail_no_consensus_state_for_height(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            chan_end_on_a_unordered,
            conn_end_on_a,
            packet_commitment,
            ..
        } = fixture;

        let packet = msg.packet.clone();

        let context = context
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
                packet_commitment,
            );

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because the client does not have a consensus state for the required height"
        )
    }

    #[rstest]
    fn timeout_fail_proof_timeout_not_reached(fixture: Fixture) {
        let Fixture {
            context,
            mut msg,
            chan_end_on_a_unordered,
            conn_end_on_a,
            client_height,
            ..
        } = fixture;

        // timeout timestamp has not reached yet
        let timeout_timestamp_on_b =
            (msg.packet.timeout_timestamp_on_b + core::time::Duration::new(10, 0)).unwrap();
        msg.packet.timeout_timestamp_on_b = timeout_timestamp_on_b;
        let packet_commitment = compute_packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let packet = msg.packet.clone();

        let mut context = context
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
                packet_commitment,
            );

        context
            .store_update_time(
                ClientId::default(),
                client_height,
                Timestamp::from_nanoseconds(5).unwrap(),
            )
            .unwrap();
        context
            .store_update_height(
                ClientId::default(),
                client_height,
                Height::new(0, 4).unwrap(),
            )
            .unwrap();

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation should fail because the timeout height was reached, but the timestamp hasn't been reached. Both the height and timestamp need to be reached for the packet to be considered timed out"
        )
    }

    /// NO-OP case
    #[rstest]
    fn timeout_success_no_packet_commitment(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_a,
            chan_end_on_a_unordered,
            ..
        } = fixture;
        let context = context
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_connection(ConnectionId::default(), conn_end_on_a);

        let res = validate(&context, &msg);

        assert!(
            res.is_ok(),
            "Validation should succeed when no packet commitment is present"
        )
    }

    #[rstest]
    fn timeout_success_unordered_channel(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            chan_end_on_a_unordered,
            conn_end_on_a,
            packet_commitment,
            client_height,
            ..
        } = fixture;

        let packet = msg.packet.clone();

        let mut context = context
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
                packet_commitment,
            );

        context
            .store_update_time(
                ClientId::default(),
                client_height,
                Timestamp::from_nanoseconds(1000).unwrap(),
            )
            .unwrap();
        context
            .store_update_height(
                ClientId::default(),
                client_height,
                Height::new(0, 5).unwrap(),
            )
            .unwrap();

        let res = validate(&context, &msg);

        assert!(res.is_ok(), "Good parameters for unordered channels")
    }

    #[rstest]
    fn timeout_success_ordered_channel(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            chan_end_on_a_ordered,
            conn_end_on_a,
            packet_commitment,
            client_height,
            ..
        } = fixture;

        let packet = msg.packet.clone();

        let mut context = context
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_ordered,
            )
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
                packet_commitment,
            );

        context
            .store_update_time(
                ClientId::default(),
                client_height,
                Timestamp::from_nanoseconds(1000).unwrap(),
            )
            .unwrap();
        context
            .store_update_height(
                ClientId::default(),
                client_height,
                Height::new(0, 4).unwrap(),
            )
            .unwrap();

        let res = validate(&context, &msg);

        assert!(res.is_ok(), "Good parameters for unordered channels")
    }
}
