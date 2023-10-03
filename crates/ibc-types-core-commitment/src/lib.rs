//! IBC client-related types.
#![no_std]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

mod prelude;

mod error;
mod path;
mod prefix;
mod proof;
mod root;

pub use error::Error;
pub use path::MerklePath;
pub use prefix::MerklePrefix;
pub use proof::MerkleProof;
pub use root::MerkleRoot;

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mock;
