//! IBC client-related types.
#![no_std]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

mod client_id;
mod client_type;
mod error;
mod height;

mod prelude;

pub mod events;
pub mod msgs;

pub use client_id::ClientId;
pub use client_type::ClientType;
pub use error::Error;
pub use height::{Height, HeightParseError};

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mock;
