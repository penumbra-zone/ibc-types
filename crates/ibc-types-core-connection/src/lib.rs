//! IBC connection-related types.
#![no_std]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

// TODO: make private
mod connection;
mod error;
mod identifier;
mod prelude;
mod version;

pub use connection::{ConnectionEnd, Counterparty, IdentifiedConnectionEnd};
pub use error::ConnectionError;
pub use identifier::{ChainId, ConnectionId};
pub use version::Version;

pub mod events;
pub mod msgs;

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mocks;
