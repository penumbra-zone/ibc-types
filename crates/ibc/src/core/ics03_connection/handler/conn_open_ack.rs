//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenAck`.

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics03_connection::error::Error;
use crate::core::ics03_connection::events::Attributes;
use crate::core::ics03_connection::handler::ConnectionResult;
use crate::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

use super::ConnectionIdState;

/// Per our convention, this message is processed on chain A.
pub(crate) fn process(
    ctx_a: &dyn ConnectionReader,
    msg: MsgConnectionOpenAck,
) -> HandlerResult<ConnectionResult, Error> {
    let mut output = HandlerOutput::builder();

    if msg.consensus_height_of_a_on_b > ctx_a.host_current_height() {
        return Err(Error::invalid_consensus_height(
            msg.consensus_height_of_a_on_b,
            ctx_a.host_current_height(),
        ));
    }

    ///////////////////////////////////////////////////////////
    // validate_self_client() verification goes here
    // See [issue](https://github.com/cosmos/ibc-rs/issues/162)
    ///////////////////////////////////////////////////////////

    let conn_end_on_a = ctx_a.connection_end(&msg.conn_id_on_a)?;
    if !(conn_end_on_a.state_matches(&State::Init)
        && conn_end_on_a.versions().contains(&msg.version))
    {
        return Err(Error::connection_mismatch(msg.conn_id_on_a));
    }

    // Proof verification.
    {
        let client_state_of_b_on_a = ctx_a.client_state(conn_end_on_a.client_id())?;
        let consensus_state_of_b_on_a =
            ctx_a.client_consensus_state(conn_end_on_a.client_id(), msg.proofs_height_on_b)?;

        let prefix_on_a = ctx_a.commitment_prefix();
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let client_id_on_a = conn_end_on_a.client_id();
        let client_id_on_b = conn_end_on_a.counterparty().client_id();

        {
            let conn_id_on_b = conn_end_on_a
                .counterparty()
                .connection_id()
                .ok_or_else(Error::invalid_counterparty)?;
            let expected_conn_end_on_b = ConnectionEnd::new(
                State::TryOpen,
                client_id_on_b.clone(),
                Counterparty::new(
                    client_id_on_a.clone(),
                    Some(msg.conn_id_on_a.clone()),
                    prefix_on_a,
                ),
                vec![msg.version.clone()],
                conn_end_on_a.delay_period(),
            );

            client_state_of_b_on_a
                .verify_connection_state(
                    msg.proofs_height_on_b,
                    prefix_on_b,
                    &msg.proof_conn_end_on_b,
                    consensus_state_of_b_on_a.root(),
                    conn_id_on_b,
                    &expected_conn_end_on_b,
                )
                .map_err(Error::verify_connection_state)?;
        }

        client_state_of_b_on_a
            .verify_client_full_state(
                msg.proofs_height_on_b,
                prefix_on_b,
                &msg.proof_client_state_of_a_on_b,
                consensus_state_of_b_on_a.root(),
                client_id_on_b,
                msg.client_state_of_a_on_b,
            )
            .map_err(|e| {
                Error::client_state_verification_failure(conn_end_on_a.client_id().clone(), e)
            })?;

        let expected_consensus_state_of_a_on_b =
            ctx_a.host_consensus_state(msg.consensus_height_of_a_on_b)?;
        client_state_of_b_on_a
            .verify_client_consensus_state(
                msg.proofs_height_on_b,
                prefix_on_b,
                &msg.proof_consensus_state_of_a_on_b,
                consensus_state_of_b_on_a.root(),
                conn_end_on_a.counterparty().client_id(),
                msg.consensus_height_of_a_on_b,
                expected_consensus_state_of_a_on_b.as_ref(),
            )
            .map_err(|e| Error::consensus_state_verification_failure(msg.proofs_height_on_b, e))?;
    }

    // Success
    let result = {
        let new_conn_end_on_a = {
            let mut counterparty = conn_end_on_a.counterparty().clone();
            counterparty.connection_id = Some(msg.conn_id_on_b.clone());

            let mut new_conn_end_on_a = conn_end_on_a;
            new_conn_end_on_a.set_state(State::Open);
            new_conn_end_on_a.set_version(msg.version.clone());
            new_conn_end_on_a.set_counterparty(counterparty);
            new_conn_end_on_a
        };

        ConnectionResult {
            connection_id: msg.conn_id_on_a,
            connection_id_state: ConnectionIdState::Reused,
            connection_end: new_conn_end_on_a,
        }
    };

    let event_attributes = Attributes {
        connection_id: Some(result.connection_id.clone()),
        ..Default::default()
    };

    output.emit(IbcEvent::OpenAckConnection(event_attributes.into()));
    output.log("success: conn_open_ack verification passed");

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use core::str::FromStr;
    use test_log::test;

    use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
    use crate::core::ics03_connection::error;
    use crate::core::ics03_connection::handler::{dispatch, ConnectionResult};
    use crate::core::ics03_connection::msgs::conn_open_ack::test_util::get_dummy_raw_msg_conn_open_ack;
    use crate::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
    use crate::core::ics03_connection::msgs::ConnectionMsg;
    use crate::core::ics23_commitment::commitment::CommitmentPrefix;
    use crate::core::ics24_host::identifier::{ChainId, ClientId};
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::mock::host::HostType;
    use crate::timestamp::ZERO_DURATION;

    #[test]
    fn conn_open_ack_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ConnectionMsg,
            want_pass: bool,
            match_error: Box<dyn FnOnce(error::Error)>,
        }

        let msg_ack =
            MsgConnectionOpenAck::try_from(get_dummy_raw_msg_conn_open_ack(10, 10)).unwrap();
        let conn_id = msg_ack.conn_id_on_a.clone();
        let counterparty_conn_id = msg_ack.conn_id_on_b.clone();

        // Client parameters -- identifier and correct height (matching the proof height)
        let client_id = ClientId::from_str("mock_clientid").unwrap();
        let proof_height = msg_ack.proofs_height_on_b;

        // Parametrize the host chain to have a height at least as recent as the
        // the height of the proofs in the Ack msg.
        let latest_height = proof_height.increment();
        let max_history_size = 5;
        let default_context = MockContext::new(
            ChainId::new("mockgaia".to_string(), latest_height.revision_number()),
            HostType::Mock,
            max_history_size,
            latest_height,
        );

        // A connection end that will exercise the successful path.
        let default_conn_end = ConnectionEnd::new(
            State::Init,
            client_id.clone(),
            Counterparty::new(
                client_id.clone(),
                Some(msg_ack.conn_id_on_b.clone()),
                CommitmentPrefix::try_from(b"ibc".to_vec()).unwrap(),
            ),
            vec![msg_ack.version.clone()],
            ZERO_DURATION,
        );

        // A connection end with incorrect state `Open`; will be part of the context.
        let mut conn_end_open = default_conn_end.clone();
        conn_end_open.set_state(State::Open); // incorrect field

        let tests: Vec<Test> = vec![
            Test {
                name: "Successful processing of an Ack message".to_string(),
                ctx: default_context
                    .clone()
                    .with_client(&client_id, proof_height)
                    .with_connection(conn_id.clone(), default_conn_end),
                msg: ConnectionMsg::ConnectionOpenAck(Box::new(msg_ack.clone())),
                want_pass: true,
                match_error: Box::new(|_| panic!("should not have error")),
            },
            Test {
                name: "Processing fails because the connection does not exist in the context"
                    .to_string(),
                ctx: default_context.clone(),
                msg: ConnectionMsg::ConnectionOpenAck(Box::new(msg_ack.clone())),
                want_pass: false,
                match_error: {
                    let connection_id = conn_id.clone();
                    Box::new(move |e| match e.detail() {
                        error::ErrorDetail::ConnectionNotFound(e) => {
                            assert_eq!(e.connection_id, connection_id)
                        }
                        _ => {
                            panic!("Expected ConnectionNotFound error");
                        }
                    })
                },
            },
            Test {
                name: "Processing fails due to connections mismatch (incorrect 'open' state)"
                    .to_string(),
                ctx: default_context
                    .with_client(&client_id, proof_height)
                    .with_connection(conn_id.clone(), conn_end_open),
                msg: ConnectionMsg::ConnectionOpenAck(Box::new(msg_ack)),
                want_pass: false,
                match_error: {
                    let connection_id = conn_id;
                    Box::new(move |e| match e.detail() {
                        error::ErrorDetail::ConnectionMismatch(e) => {
                            assert_eq!(e.connection_id, connection_id);
                        }
                        _ => {
                            panic!("Expected ConnectionMismatch error");
                        }
                    })
                },
            },
            /*
            Test {
                name: "Processing fails due to MissingLocalConsensusState".to_string(),
                ctx: MockContext::default()
                    .with_client(&client_id, proof_height)
                    .with_connection(conn_id, default_conn_end),
                msg: ConnectionMsg::ConnectionOpenAck(Box::new(msg_ack)),
                want_pass: false,
                error_kind: Some(Kind::MissingLocalConsensusState)
            },
            */
        ];

        for test in tests {
            let res = dispatch(&test.ctx, test.msg.clone());
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(
                        test.want_pass,
                        "conn_open_ack: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    // The object in the output is a ConnectionEnd, should have OPEN state.
                    let res: ConnectionResult = proto_output.result;
                    assert_eq!(res.connection_end.state().clone(), State::Open);

                    // assert that counterparty connection id is correct
                    assert_eq!(
                        res.connection_end.counterparty().connection_id,
                        Some(counterparty_conn_id.clone())
                    );

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::OpenAckConnection(_)));
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "conn_open_ack: failed for test: {}, \nparams {:?} {:?} error: {:?}",
                        test.name,
                        test.msg,
                        test.ctx.clone(),
                        e,
                    );

                    // Verify that the error kind matches
                    (test.match_error)(e);
                }
            }
        }
    }
}