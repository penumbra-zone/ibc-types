pub use core::prelude::v1::*;

// allow `unused_imports`, rustc errantly claims this `vec!` is not used.
#[allow(unused_imports)]
// Re-export according to alloc::prelude::v1 because it is not yet stabilized
// https://doc.rust-lang.org/src/alloc/prelude/v1.rs.html
pub use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

pub use alloc::format;
