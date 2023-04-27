//! ICS 02: Client implementation for verifying remote IBC-enabled chains.
#![no_std]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod client_id;
pub mod client_type;
pub mod error;
pub mod events;
pub mod height;
pub mod msgs;
pub mod trust_threshold;

mod prelude;

pub use client_id::ClientId;
pub use client_type::ClientType;
pub use height::{Height, HeightError};

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mock;
