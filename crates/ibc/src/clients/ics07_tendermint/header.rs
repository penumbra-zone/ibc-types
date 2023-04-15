use alloc::string::ToString;
use core::fmt::{Display, Error as FmtError, Formatter};

use bytes::Buf;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
use ibc_proto::protobuf::Protobuf;
use prost::Message;
use tendermint::block::signed_header::SignedHeader;
use tendermint::chain::Id as TmChainId;
use tendermint::validator::Set as ValidatorSet;
use tendermint_light_client_verifier::types::{TrustedBlockState, UntrustedBlockState};

use crate::clients::ics07_tendermint::consensus_state::ConsensusState;
use crate::clients::ics07_tendermint::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::ChainId;
use crate::timestamp::Timestamp;
use crate::utils::pretty::{PrettySignedHeader, PrettyValidatorSet};
use crate::Height;

pub const TENDERMINT_HEADER_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.Header";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tendermint consensus header
#[derive(Clone, PartialEq, Eq)]
pub struct Header {
    pub signed_header: SignedHeader, // contains the commitment root
    pub validator_set: ValidatorSet, // the validator set that signed Header
    pub trusted_height: Height, // the height of a trusted header seen by client less than or equal to Header
    pub trusted_next_validator_set: ValidatorSet, // the last trusted validator set at trusted height
}

impl core::fmt::Debug for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, " Header {{...}}")
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "Header {{ signed_header: {}, validator_set: {}, trusted_height: {}, trusted_validator_set: {} }}", PrettySignedHeader(&self.signed_header), PrettyValidatorSet(&self.validator_set), self.trusted_height, PrettyValidatorSet(&self.trusted_next_validator_set))
    }
}

impl Header {
    pub fn height(&self) -> Height {
        Height::new(
            ChainId::chain_version(self.signed_header.header.chain_id.as_str()),
            u64::from(self.signed_header.header.height),
        )
        .expect("malformed tendermint header domain type has an illegal height of 0")
    }

    pub(crate) fn as_untrusted_block_state(&self) -> UntrustedBlockState<'_> {
        UntrustedBlockState {
            signed_header: &self.signed_header,
            validators: &self.validator_set,
            next_validators: None,
        }
    }

    pub(crate) fn as_trusted_block_state<'a>(
        &'a self,
        consensus_state: &ConsensusState,
        chain_id: &'a TmChainId,
    ) -> Result<TrustedBlockState<'a>, Error> {
        Ok(TrustedBlockState {
            chain_id,
            header_time: consensus_state.timestamp,
            height: self
                .trusted_height
                .revision_height()
                .try_into()
                .map_err(|_| Error::InvalidHeaderHeight {
                    height: self.trusted_height.revision_height(),
                })?,
            next_validators: &self.trusted_next_validator_set,
            next_validators_hash: consensus_state.next_validators_hash,
        })
    }
}

impl crate::core::ics02_client::header::Header for Header {
    fn height(&self) -> Height {
        self.height()
    }

    fn timestamp(&self) -> Timestamp {
        self.signed_header.header.time.into()
    }
}

impl Protobuf<RawHeader> for Header {}

impl TryFrom<RawHeader> for Header {
    type Error = Error;

    fn try_from(raw: RawHeader) -> Result<Self, Self::Error> {
        let header = Self {
            signed_header: raw
                .signed_header
                .ok_or(Error::MissingSignedHeader)?
                .try_into()
                .map_err(|e| Error::InvalidHeader {
                    reason: "signed header conversion".to_string(),
                    error: e,
                })?,
            validator_set: raw
                .validator_set
                .ok_or(Error::MissingValidatorSet)?
                .try_into()
                .map_err(Error::InvalidRawHeader)?,
            trusted_height: raw
                .trusted_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(Error::MissingTrustedHeight)?,
            trusted_next_validator_set: raw
                .trusted_validators
                .ok_or(Error::MissingTrustedNextValidatorSet)?
                .try_into()
                .map_err(Error::InvalidRawHeader)?,
        };

        if header.height().revision_number() != header.trusted_height.revision_number() {
            return Err(Error::MismatchedRevisions {
                current_revision: header.trusted_height.revision_number(),
                update_revision: header.height().revision_number(),
            });
        }

        Ok(header)
    }
}

impl Protobuf<Any> for Header {}

impl TryFrom<Any> for Header {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use core::ops::Deref;

        match raw.type_url.as_str() {
            TENDERMINT_HEADER_TYPE_URL => decode_header(raw.value.deref()).map_err(Into::into),
            _ => Err(ClientError::UnknownHeaderType {
                header_type: raw.type_url,
            }),
        }
    }
}

impl From<Header> for Any {
    fn from(header: Header) -> Self {
        Any {
            type_url: TENDERMINT_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<RawHeader>::encode_vec(&header),
        }
    }
}

pub fn decode_header<B: Buf>(buf: B) -> Result<Header, Error> {
    RawHeader::decode(buf).map_err(Error::Decode)?.try_into()
}

impl From<Header> for RawHeader {
    fn from(value: Header) -> Self {
        RawHeader {
            signed_header: Some(value.signed_header.into()),
            validator_set: Some(value.validator_set.into()),
            trusted_height: Some(value.trusted_height.into()),
            trusted_validators: Some(value.trusted_next_validator_set.into()),
        }
    }
}

#[cfg(any(test, feature = "mocks"))]
pub mod test_util {
    use alloc::vec;

    use subtle_encoding::hex;
    use tendermint::block::signed_header::SignedHeader;
    use tendermint::validator::Info as ValidatorInfo;
    use tendermint::validator::Set as ValidatorSet;
    use tendermint::PublicKey;

    use crate::clients::ics07_tendermint::header::Header;
    use crate::mock::host::SyntheticTmBlock;
    use crate::Height;

    pub fn get_dummy_tendermint_header() -> tendermint::block::Header {
        serde_json::from_str::<SignedHeader>(include_str!(
            "../../../tests/support/signed_header.json"
        ))
        .unwrap()
        .header
    }

    // TODO: This should be replaced with a ::default() or ::produce().
    // The implementation of this function comprises duplicate code (code borrowed from
    // `tendermint-rs` for assembling a Header).
    // See https://github.com/informalsystems/tendermint-rs/issues/381.
    //
    // The normal flow is:
    // - get the (trusted) signed header and the `trusted_validator_set` at a `trusted_height`
    // - get the `signed_header` and the `validator_set` at latest height
    // - build the ics07 Header
    // For testing purposes this function does:
    // - get the `signed_header` from a .json file
    // - create the `validator_set` with a single validator that is also the proposer
    // - assume a `trusted_height` of 1 and no change in the validator set since height 1,
    //   i.e. `trusted_validator_set` = `validator_set`
    pub fn get_dummy_ics07_header() -> Header {
        // Build a SignedHeader from a JSON file.
        let shdr = serde_json::from_str::<SignedHeader>(include_str!(
            "../../../tests/support/signed_header.json"
        ))
        .unwrap();

        // Build a set of validators.
        // Below are test values inspired form `test_validator_set()` in tendermint-rs.
        let v1: ValidatorInfo = ValidatorInfo::new(
            PublicKey::from_raw_ed25519(
                &hex::decode_upper(
                    "F349539C7E5EF7C49549B09C4BFC2335318AB0FE51FBFAA2433B4F13E816F4A7",
                )
                .unwrap(),
            )
            .unwrap(),
            281_815_u64.try_into().unwrap(),
        );

        let vs = ValidatorSet::new(vec![v1.clone()], Some(v1));

        Header {
            signed_header: shdr,
            validator_set: vs.clone(),
            trusted_height: Height::new(0, 1).unwrap(),
            trusted_next_validator_set: vs,
        }
    }

    impl From<SyntheticTmBlock> for Header {
        fn from(light_block: SyntheticTmBlock) -> Self {
            let SyntheticTmBlock {
                trusted_height,
                light_block,
            } = light_block;
            Self {
                signed_header: light_block.signed_header,
                validator_set: light_block.validators,
                trusted_height,
                trusted_next_validator_set: light_block.next_validators,
            }
        }
    }
}
