use crate::prelude::*;
use core::fmt::{Display, Error as FmtError, Formatter};

/// Type of the client, depending on the specific consensus algorithm.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientType(pub String);

impl ClientType {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// Yields this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ClientType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}
