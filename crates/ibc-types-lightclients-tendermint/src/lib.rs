//! ICS 07: Tendermint Client implements a client verification algorithm for blockchains which use
//! the Tendermint consensus algorithm.

// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;

use alloc::string::ToString;

use ibc_types_core_client::ClientType;

mod prelude;

mod trust_threshold;
pub use trust_threshold::TrustThreshold;

mod error;
pub use error::{Error, VerificationError};

pub mod client_state;
pub mod consensus_state;
pub mod header;
pub mod misbehaviour;

pub use consensus_state::ConsensusState;

pub const TENDERMINT_CLIENT_TYPE: &str = "07-tendermint";

pub fn client_type() -> ClientType {
    ClientType::new(TENDERMINT_CLIENT_TYPE.to_string())
}
