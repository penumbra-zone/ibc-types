use crate::prelude::*;

use core::convert::{TryFrom, TryInto};
use core::time::Duration;
use std::string::String;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;

use ibc_proto::ibc::lightclients::tendermint::v1::ClientState as RawTmClientState;
use ibc_proto::protobuf::Protobuf;
use ibc_types_domain_type::{DomainType, TypeUrl};
use ics23::ProofSpec;
use prost::Message;
use tendermint::chain::id::MAX_LENGTH as MaxChainIdLen;
use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;
use tendermint_light_client_verifier::options::Options;

use crate::header::Header as TmHeader;

use ibc_types_core_client::Height;

use ibc_types_core_connection::ChainId;
use ibc_types_timestamp::Timestamp;

use crate::{Error, TrustThreshold};

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";

#[derive(Clone, Debug, PartialEq)]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trust_level: TrustThreshold,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
    pub latest_height: Height,
    pub proof_specs: Vec<ProofSpec>,
    pub upgrade_path: Vec<String>,
    pub allow_update: AllowUpdate,
    pub frozen_height: Option<Height>,
}

impl TypeUrl for ClientState {
    const TYPE_URL: &'static str = TENDERMINT_CLIENT_STATE_TYPE_URL;
}

impl DomainType for ClientState {
    type Proto = Any;
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllowUpdate {
    pub after_expiry: bool,
    pub after_misbehaviour: bool,
}

impl ClientState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: Vec<ProofSpec>,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
        frozen_height: Option<Height>,
    ) -> Result<ClientState, Error> {
        if chain_id.as_str().len() > MaxChainIdLen {
            return Err(Error::ChainIdTooLong {
                chain_id: chain_id.clone(),
                len: chain_id.as_str().len(),
                max_len: MaxChainIdLen,
            });
        }

        // `TrustThreshold` is guaranteed to be in the range `[0, 1)`, but a `TrustThreshold::ZERO`
        // value is invalid in this context
        if trust_level == TrustThreshold::ZERO {
            return Err(Error::InvalidTrustThreshold {
                reason: "ClientState trust-level cannot be zero".to_string(),
            });
        }

        let _ = TendermintTrustThresholdFraction::new(
            trust_level.numerator(),
            trust_level.denominator(),
        )
        .map_err(Error::InvalidTendermintTrustThreshold)?;

        // Basic validation of trusting period and unbonding period: each should be non-zero.
        if trusting_period <= Duration::new(0, 0) {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                    "ClientState trusting period ({trusting_period:?}) must be greater than zero"
                ),
            });
        }

        if unbonding_period <= Duration::new(0, 0) {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                    "ClientState unbonding period ({unbonding_period:?}) must be greater than zero"
                ),
            });
        }

        if trusting_period >= unbonding_period {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                "ClientState trusting period ({trusting_period:?}) must be smaller than unbonding period ({unbonding_period:?})"
            ),
            });
        }

        if max_clock_drift <= Duration::new(0, 0) {
            return Err(Error::InvalidMaxClockDrift {
                reason: "ClientState max-clock-drift must be greater than zero".to_string(),
            });
        }

        if latest_height.revision_number() != chain_id.version() {
            return Err(Error::InvalidLatestHeight {
                reason: "ClientState latest-height revision number must match chain-id version"
                    .to_string(),
            });
        }

        // Disallow empty proof-specs
        if proof_specs.is_empty() {
            return Err(Error::Validation {
                reason: "ClientState proof-specs cannot be empty".to_string(),
            });
        }

        // `upgrade_path` itself may be empty, but if not then each key must be non-empty
        for (idx, key) in upgrade_path.iter().enumerate() {
            if key.trim().is_empty() {
                return Err(Error::Validation {
                    reason: format!(
                        "ClientState upgrade-path key at index {idx:?} cannot be empty"
                    ),
                });
            }
        }

        Ok(Self {
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            proof_specs,
            upgrade_path,
            allow_update,
            frozen_height,
        })
    }

    pub fn latest_height(&self) -> Height {
        self.latest_height
    }

    /// returns a bool indicating if the client is frozen, i.e. if a frozen height is set.
    pub fn is_frozen(&self) -> bool {
        self.frozen_height.is_some()
    }

    pub fn expired(&self, elapsed: Duration) -> bool {
        elapsed > self.trusting_period
    }

    pub fn with_header(self, h: TmHeader) -> Result<Self, Error> {
        Ok(ClientState {
            latest_height: Height::new(
                self.latest_height.revision_number(),
                h.signed_header.header.height.into(),
            )
            .map_err(|_| Error::InvalidHeaderHeight {
                height: h.signed_header.header.height.value(),
            })?,
            ..self
        })
    }

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            frozen_height: Some(h),
            ..self
        }
    }

    /// Get the refresh time to ensure the state does not expire
    pub fn refresh_time(&self) -> Option<Duration> {
        Some(2 * self.trusting_period / 3)
    }

    /// Helper method to produce a [`Options`] struct for use in
    /// Tendermint-specific light client verification.
    pub fn as_light_client_options(&self) -> Result<Options, Error> {
        Ok(Options {
            trust_threshold: self.trust_level.try_into().map_err(|e: Error| {
                Error::InvalidTrustThreshold {
                    reason: e.to_string(),
                }
            })?,
            trusting_period: self.trusting_period,
            clock_drift: self.max_clock_drift,
        })
    }

    /// Verify the time and height delays
    pub fn verify_delay_passed(
        current_time: Timestamp,
        current_height: Height,
        processed_time: Timestamp,
        processed_height: Height,
        delay_period_time: Duration,
        delay_period_blocks: u64,
    ) -> Result<(), Error> {
        let earliest_time =
            (processed_time + delay_period_time).map_err(Error::TimestampOverflow)?;
        if !(current_time == earliest_time || current_time.after(&earliest_time)) {
            return Err(Error::NotEnoughTimeElapsed {
                current_time,
                earliest_time,
            });
        }

        let earliest_height = processed_height.add(delay_period_blocks);
        if current_height < earliest_height {
            return Err(Error::NotEnoughBlocksElapsed {
                current_height,
                earliest_height,
            });
        }

        Ok(())
    }

    /// Verify that the client is at a sufficient height and unfrozen at the given height
    pub fn verify_height(&self, height: Height) -> Result<(), Error> {
        if self.latest_height < height {
            return Err(Error::InsufficientHeight {
                latest_height: self.latest_height(),
                target_height: height,
            });
        }

        match self.frozen_height {
            Some(frozen_height) if frozen_height <= height => Err(Error::ClientFrozen {
                frozen_height,
                target_height: height,
            }),
            _ => Ok(()),
        }
    }
}

impl Protobuf<RawTmClientState> for ClientState {}

impl TryFrom<RawTmClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawTmClientState) -> Result<Self, Self::Error> {
        let chain_id = ChainId::from_string(raw.chain_id.as_str());

        let trust_level = {
            let trust_level = raw
                .trust_level
                .clone()
                .ok_or(Error::MissingTrustingPeriod)?;
            trust_level
                .try_into()
                .map_err(|e| Error::InvalidTrustThreshold {
                    reason: format!("{e}"),
                })?
        };

        let trusting_period = raw
            .trusting_period
            .ok_or(Error::MissingTrustingPeriod)?
            .try_into()
            .map_err(|_| Error::MissingTrustingPeriod)?;

        let unbonding_period = raw
            .unbonding_period
            .ok_or(Error::MissingUnbondingPeriod)?
            .try_into()
            .map_err(|_| Error::MissingUnbondingPeriod)?;

        let max_clock_drift = raw
            .max_clock_drift
            .ok_or(Error::NegativeMaxClockDrift)?
            .try_into()
            .map_err(|_| Error::NegativeMaxClockDrift)?;

        let latest_height = raw
            .latest_height
            .ok_or(Error::MissingLatestHeight)?
            .try_into()
            .map_err(|_| Error::MissingLatestHeight)?;

        // In `RawClientState`, a `frozen_height` of `0` means "not frozen".
        // See:
        // https://github.com/cosmos/ibc-go/blob/8422d0c4c35ef970539466c5bdec1cd27369bab3/modules/light-clients/07-tendermint/types/client_state.go#L74
        let frozen_height = raw
            .frozen_height
            .and_then(|raw_height| raw_height.try_into().ok());

        // We use set this deprecated field just so that we can properly convert
        // it back in its raw form
        #[allow(deprecated)]
        let allow_update = AllowUpdate {
            after_expiry: raw.allow_update_after_expiry,
            after_misbehaviour: raw.allow_update_after_misbehaviour,
        };

        let client_state = ClientState::new(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            raw.proof_specs.into(),
            raw.upgrade_path,
            allow_update,
            frozen_height,
        )?;

        Ok(client_state)
    }
}

impl From<ClientState> for RawTmClientState {
    fn from(value: ClientState) -> Self {
        #[allow(deprecated)]
        Self {
            chain_id: value.chain_id.to_string(),
            trust_level: Some(value.trust_level.into()),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
            frozen_height: Some(value.frozen_height.map(|height| height.into()).unwrap_or(
                RawHeight {
                    revision_number: 0,
                    revision_height: 0,
                },
            )),
            latest_height: Some(value.latest_height.into()),
            proof_specs: value.proof_specs.into(),
            upgrade_path: value.upgrade_path,
            allow_update_after_expiry: value.allow_update.after_expiry,
            allow_update_after_misbehaviour: value.allow_update.after_misbehaviour,
        }
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use bytes::Buf;
        use core::ops::Deref;

        fn decode_client_state<B: Buf>(buf: B) -> Result<ClientState, Error> {
            RawTmClientState::decode(buf)
                .map_err(Error::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => {
                decode_client_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(Error::WrongTypeUrl { url: raw.type_url }),
        }
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawTmClientState>::encode_vec(&client_state),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use core::time::Duration;
    use test_log::test;

    use ibc_types_core_client::Height;
    use ibc_types_core_connection::ChainId;
    use ibc_types_timestamp::{Timestamp, ZERO_DURATION};

    use super::*;

    use crate::TrustThreshold;

    use ics23::ProofSpec;

    #[derive(Clone, Debug, PartialEq)]
    struct ClientStateParams {
        id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: Vec<ProofSpec>,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
    }

    #[test]
    fn client_state_new() {
        // Define a "default" set of parameters to reuse throughout these tests.
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::default(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(0, 10).unwrap(),
            proof_specs: vec![ics23::iavl_spec()],
            upgrade_path: Default::default(),
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };

        struct Test {
            name: String,
            params: ClientStateParams,
            want_pass: bool,
        }

        let tests: Vec<Test> = vec![
            Test {
                name: "Valid parameters".to_string(),
                params: default_params.clone(),
                want_pass: true,
            },
            Test {
                name: "Valid (empty) upgrade-path".to_string(),
                params: ClientStateParams {
                    upgrade_path: vec![],
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Valid upgrade-path".to_string(),
                params: ClientStateParams {
                    upgrade_path: vec!["upgrade".to_owned(), "upgradedIBCState".to_owned()],
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Valid long (50 chars) chain-id".to_string(),
                params: ClientStateParams {
                    id: ChainId::new("a".repeat(48), 0),
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Invalid too-long (51 chars) chain-id".to_string(),
                params: ClientStateParams {
                    id: ChainId::new("a".repeat(49), 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (zero) max-clock-drift period".to_string(),
                params: ClientStateParams {
                    max_clock_drift: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid unbonding period".to_string(),
                params: ClientStateParams {
                    unbonding_period: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too small) trusting period".to_string(),
                params: ClientStateParams {
                    trusting_period: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too large) trusting period w.r.t. unbonding period".to_string(),
                params: ClientStateParams {
                    trusting_period: Duration::new(11, 0),
                    unbonding_period: Duration::new(10, 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (equal) trusting period w.r.t. unbonding period".to_string(),
                params: ClientStateParams {
                    trusting_period: Duration::new(10, 0),
                    unbonding_period: Duration::new(10, 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (zero) trusting trust threshold".to_string(),
                params: ClientStateParams {
                    trust_level: TrustThreshold::ZERO,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too small) trusting trust threshold".to_string(),
                params: ClientStateParams {
                    trust_level: TrustThreshold::new(1, 4).unwrap(),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid latest height revision number (doesn't match chain)".to_string(),
                params: ClientStateParams {
                    latest_height: Height::new(1, 1).unwrap(),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (empty) proof specs".to_string(),
                params: ClientStateParams {
                    proof_specs: Vec::new(),
                    ..default_params
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let p = test.params.clone();

            let cs_result = ClientState::new(
                p.id,
                p.trust_level,
                p.trusting_period,
                p.unbonding_period,
                p.max_clock_drift,
                p.latest_height,
                p.proof_specs,
                p.upgrade_path,
                p.allow_update,
                None,
            );

            assert_eq!(
                test.want_pass,
                cs_result.is_ok(),
                "ClientState::new() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.params.clone(),
                cs_result.err(),
            );
        }
    }

    #[test]
    fn client_state_verify_delay_passed() {
        #[derive(Debug, Clone)]
        struct Params {
            current_time: Timestamp,
            current_height: Height,
            processed_time: Timestamp,
            processed_height: Height,
            delay_period_time: Duration,
            delay_period_blocks: u64,
        }
        struct Test {
            name: String,
            params: Params,
            want_pass: bool,
        }
        let now = Timestamp::now();

        let tests: Vec<Test> = vec![
            Test {
                name: "Successful delay verification".to_string(),
                params: Params {
                    current_time: (now + Duration::from_nanos(2000)).unwrap(),
                    current_height: Height::new(0, 5).unwrap(),
                    processed_time: (now + Duration::from_nanos(1000)).unwrap(),
                    processed_height: Height::new(0, 3).unwrap(),
                    delay_period_time: Duration::from_nanos(500),
                    delay_period_blocks: 2,
                },
                want_pass: true,
            },
            Test {
                name: "Delay period(time) has not elapsed".to_string(),
                params: Params {
                    current_time: (now + Duration::from_nanos(1200)).unwrap(),
                    current_height: Height::new(0, 5).unwrap(),
                    processed_time: (now + Duration::from_nanos(1000)).unwrap(),
                    processed_height: Height::new(0, 3).unwrap(),
                    delay_period_time: Duration::from_nanos(500),
                    delay_period_blocks: 2,
                },
                want_pass: false,
            },
            Test {
                name: "Delay period(blocks) has not elapsed".to_string(),
                params: Params {
                    current_time: (now + Duration::from_nanos(2000)).unwrap(),
                    current_height: Height::new(0, 5).unwrap(),
                    processed_time: (now + Duration::from_nanos(1000)).unwrap(),
                    processed_height: Height::new(0, 4).unwrap(),
                    delay_period_time: Duration::from_nanos(500),
                    delay_period_blocks: 2,
                },
                want_pass: false,
            },
        ];

        for test in tests {
            let res = ClientState::verify_delay_passed(
                test.params.current_time,
                test.params.current_height,
                test.params.processed_time,
                test.params.processed_height,
                test.params.delay_period_time,
                test.params.delay_period_blocks,
            );

            assert_eq!(
                test.want_pass,
                res.is_ok(),
                "ClientState::verify_delay_passed() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.params.clone(),
                res.err(),
            );
        }
    }

    #[test]
    fn client_state_verify_height() {
        // Define a "default" set of parameters to reuse throughout these tests.
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::new("ibc".to_string(), 1),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(1, 10).unwrap(),
            proof_specs: vec![ics23::iavl_spec()],
            upgrade_path: Default::default(),
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };

        struct Test {
            name: String,
            height: Height,
            setup: Option<Box<dyn FnOnce(ClientState) -> ClientState>>,
            want_pass: bool,
        }

        let tests = vec![
            Test {
                name: "Successful height verification".to_string(),
                height: Height::new(1, 8).unwrap(),
                setup: None,
                want_pass: true,
            },
            Test {
                name: "Invalid (too large)  client height".to_string(),
                height: Height::new(1, 12).unwrap(),
                setup: None,
                want_pass: false,
            },
            Test {
                name: "Invalid, client is frozen below current height".to_string(),
                height: Height::new(1, 6).unwrap(),
                setup: Some(Box::new(|client_state| {
                    client_state.with_frozen_height(Height::new(1, 5).unwrap())
                })),
                want_pass: false,
            },
        ];

        for test in tests {
            let p = default_params.clone();
            let client_state = ClientState::new(
                p.id,
                p.trust_level,
                p.trusting_period,
                p.unbonding_period,
                p.max_clock_drift,
                p.latest_height,
                p.proof_specs,
                p.upgrade_path,
                p.allow_update,
                None,
            )
            .unwrap();
            let client_state = match test.setup {
                Some(setup) => (setup)(client_state),
                _ => client_state,
            };
            let res = client_state.verify_height(test.height);

            assert_eq!(
                test.want_pass,
                res.is_ok(),
                "ClientState::verify_delay_height() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.height,
                res.err(),
            );
        }
    }
}

/*
#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use tendermint_rpc::endpoint::abci_query::AbciQuery;

    use crate::test::test_serialization_roundtrip;

    #[test]
    fn serialization_roundtrip_no_proof() {
        let json_data =
            include_str!("../../../tests/support/query/serialization/client_state.json");
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }

    #[test]
    fn serialization_roundtrip_with_proof() {
        let json_data =
            include_str!("../../../tests/support/query/serialization/client_state_proof.json");
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }
}
*/

#[cfg(any(test, feature = "mocks"))]
pub mod test_util {
    use crate::prelude::*;
    use core::time::Duration;

    use tendermint::block::Header;

    use super::*;

    use ibc_types_core_client::Height;
    use ibc_types_core_connection::ChainId;

    pub fn get_dummy_tendermint_client_state(tm_header: Header) -> ClientState {
        ClientState::new(
            ChainId::from(tm_header.chain_id.clone()),
            Default::default(),
            Duration::from_secs(64000),
            Duration::from_secs(128000),
            Duration::from_millis(3000),
            Height::new(
                ChainId::chain_version(tm_header.chain_id.as_str()),
                u64::from(tm_header.height),
            )
            .unwrap(),
            Default::default(),
            Default::default(),
            AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
            None,
        )
        .unwrap()
    }
}
