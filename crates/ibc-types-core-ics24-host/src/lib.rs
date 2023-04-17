//! ICS 24: Host defines the minimal set of interfaces that a
//! state machine hosting an IBC-enabled chain must implement.

#![no_std]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;
mod prelude;

pub use path::{ClientUpgradePath, Path, IBC_QUERY_PATH, SDK_UPGRADE_QUERY_PATH};

pub mod error;
pub mod identifier;
pub mod path;
pub mod validate;
