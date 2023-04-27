//! IBC client-related types.
#![no_std]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

mod client_id;
mod client_type;
mod error;
mod height;
mod trust_threshold;

mod prelude;

pub mod events;
pub mod msgs;

pub use client_id::{ClientId, ClientIdParseError};
pub use client_type::ClientType;
pub use error::Error;
pub use height::{Height, HeightParseError};
// TODO: TrustThreshold is a domain type for a tendermint-light-client proto type, does it belong here?
// it's not used elsewhere in the crate...
pub use trust_threshold::TrustThreshold;

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mock;
