use crate::{prelude::*, ChannelId, PortId, Version};

use ibc_types_core_connection::ConnectionId;
use tendermint::abci::Event as AbciEvent;

use super::channel::*;

#[test]
fn ibc_to_abci_channel_events() {
    struct Test {
        kind: &'static str,
        event: AbciEvent,
        expected_keys: Vec<&'static str>,
        expected_values: Vec<&'static str>,
    }

    let port_id = PortId::transfer();
    let channel_id = ChannelId::new(0);
    let connection_id = ConnectionId::new(0);
    let counterparty_port_id = PortId::transfer();
    let counterparty_channel_id = ChannelId::new(1);
    let version = Version::new("ics20-1".to_string());
    let expected_keys = vec![
        "port_id",
        "channel_id",
        "counterparty_port_id",
        "counterparty_channel_id",
        "connection_id",
        "version",
    ];
    let expected_values = vec![
        "transfer",
        "channel-0",
        "transfer",
        "channel-1",
        "connection-0",
        "ics20-1",
    ];

    let tests: Vec<Test> = vec![
        Test {
            kind: "channel_open_init",
            event: OpenInit {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: counterparty_port_id.clone(),
                connection_id: connection_id.clone(),
                version: version.clone(),
            }
            .into(),
            expected_keys: vec![
                "port_id",
                "channel_id",
                "counterparty_port_id",
                "connection_id",
                "version",
            ],
            expected_values: vec![
                "transfer",
                "channel-0",
                "transfer",
                "connection-0",
                "ics20-1",
            ],
        },
        Test {
            kind: "channel_open_try",
            event: OpenTry {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: counterparty_port_id.clone(),
                counterparty_channel_id: counterparty_channel_id.clone(),
                connection_id: connection_id.clone(),
                version: version.clone(),
            }
            .into(),
            expected_keys: expected_keys.clone(),
            expected_values: expected_values.clone(),
        },
        Test {
            kind: "channel_open_ack",
            event: OpenAck {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: counterparty_port_id.clone(),
                counterparty_channel_id: counterparty_channel_id.clone(),
                connection_id: connection_id.clone(),
            }
            .into(),
            expected_keys: expected_keys[0..5].to_vec(),
            expected_values: expected_values[0..5].to_vec(),
        },
        Test {
            kind: "channel_open_confirm",
            event: OpenConfirm {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: counterparty_port_id.clone(),
                counterparty_channel_id: counterparty_channel_id.clone(),
                connection_id: connection_id.clone(),
            }
            .into(),
            expected_keys: expected_keys[0..5].to_vec(),
            expected_values: expected_values[0..5].to_vec(),
        },
        Test {
            kind: "channel_close_init",
            event: CloseInit {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: counterparty_port_id.clone(),
                counterparty_channel_id: counterparty_channel_id.clone(),
                connection_id: connection_id.clone(),
            }
            .into(),
            expected_keys: expected_keys[0..5].to_vec(),
            expected_values: expected_values[0..5].to_vec(),
        },
        Test {
            kind: "channel_close_confirm",
            event: CloseConfirm {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: counterparty_port_id.clone(),
                counterparty_channel_id: counterparty_channel_id.clone(),
                connection_id: connection_id.clone(),
            }
            .into(),
            expected_keys: expected_keys[0..5].to_vec(),
            expected_values: expected_values[0..5].to_vec(),
        },
    ];

    for t in tests {
        assert_eq!(t.kind, t.event.kind);
        assert_eq!(t.expected_keys.len(), t.event.attributes.len());
        for (i, e) in t.event.attributes.iter().enumerate() {
            assert_eq!(
                e.key_bytes(),
                t.expected_keys[i].as_bytes(),
                "key mismatch for {:?}",
                t.kind,
            );
        }
        assert_eq!(t.expected_values.len(), t.event.attributes.len());
        for (i, e) in t.event.attributes.iter().enumerate() {
            assert_eq!(
                e.value_bytes(),
                t.expected_values[i].as_bytes(),
                "value mismatch for {:?}",
                t.kind,
            );
        }
    }
}
