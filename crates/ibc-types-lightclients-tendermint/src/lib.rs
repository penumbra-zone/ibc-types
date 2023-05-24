//! ICS 07: Tendermint Client implements a client verification algorithm for blockchains which use
//! the Tendermint consensus algorithm.
extern crate alloc;

use alloc::string::ToString;

use ibc_types_core_client::ClientType;

mod prelude;

pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod header;
pub mod misbehaviour;

pub const TENDERMINT_CLIENT_TYPE: &str = "07-tendermint";

pub fn client_type() -> ClientType {
    ClientType::new(TENDERMINT_CLIENT_TYPE.to_string())
}
