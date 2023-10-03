//! IBC connection-related types.
#![no_std]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

mod connection;
mod error;
mod identifier;
mod prelude;
mod version;

pub use connection::{ClientPaths, ConnectionEnd, Counterparty, IdentifiedConnectionEnd, State};
pub use error::ConnectionError;
pub use identifier::{ChainId, ConnectionId};
pub use version::Version;

pub mod events;
pub mod msgs;

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mocks;
