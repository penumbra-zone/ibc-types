// TODO: disable unwraps:
// https://github.com/informalsystems/ibc-rs/issues/987
// #![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![no_std]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]
// https://github.com/cosmos/ibc-rs/issues/342
#![allow(clippy::result_large_err)]
//! This library provides data types for the InterBlockchain Communication (IBC) protocol in Rust.
//!
//! This crate will eventually be a minimal implementation just providing IBC data types.  Currently, it's undergoing refactoring post-forking.

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod prelude;

pub mod applications;
pub mod clients;
pub mod core;
pub mod dynamic_typing;
mod erased;
pub mod events;
pub mod handler;
pub mod hosts;
pub mod signer;
pub mod timestamp;
pub mod tx_msg;
pub mod utils;

#[cfg(feature = "serde")]
mod serializers;

/// Re-export of ICS 002 Height domain type
pub type Height = crate::core::ics02_client::height::Height;

#[cfg(test)]
mod test;

#[cfg(any(test, feature = "mocks"))]
pub mod test_utils;

pub mod mock; // Context mock, the underlying host chain, and client types: for testing all handlers.
