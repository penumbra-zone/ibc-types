pub use core::prelude::v1::*;

// Re-export according to alloc::prelude::v1 because it is not yet stabilized
// https://doc.rust-lang.org/src/alloc/prelude/v1.rs.html
pub use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
};

pub use alloc::{format, vec};
