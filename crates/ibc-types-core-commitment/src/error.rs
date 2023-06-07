use crate::prelude::*;

use displaydoc::Display;

/// A catch-all error type.
#[derive(Debug, Display)]
pub enum Error {}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            _ => None,
        }
    }
}
