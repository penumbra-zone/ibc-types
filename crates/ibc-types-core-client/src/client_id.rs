use core::{
    fmt::{Debug, Display, Error as FmtError, Formatter},
    str::FromStr,
};

use derive_more::Into;
use ibc_types_identifier::{validate_client_identifier, IdentifierError};

use crate::{client_type::ClientType, prelude::*};

/// An IBC client identifier.
///
/// Client identifiers are deterministically formed from two elements: a prefix
/// derived from the client type `ctype`, and a monotonically increasing
/// `counter`; these are separated by a dash "-".
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Into)]
pub struct ClientId(pub(crate) String);

impl ClientId {
    /// Construct a new client identifier from a client type and a counter.
    ///
    /// ```
    /// # use ibc_types_core_client::{ClientId, ClientType};
    /// let tm_client_id = ClientId::new(ClientType::new("07-tendermint".to_string()), 0);
    /// assert!(tm_client_id.is_ok());
    /// tm_client_id.map(|id| { assert_eq!(&id, "07-tendermint-0") });
    /// ```
    pub fn new(client_type: ClientType, counter: u64) -> Result<Self, IdentifierError> {
        let prefix = client_type.as_str();
        let id = format!("{prefix}-{counter}");
        Self::from_str(id.as_str())
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    // TODO: add accessors for counter, client type
}

/// This implementation provides a `to_string` method.
impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ClientId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_client_identifier(s).map(|_| Self(s.to_string()))
    }
}

impl TryFrom<String> for ClientId {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate_client_identifier(&value).map(|_| Self(value))
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self("07-tendermint-0".to_string())
    }
}

/// Equality check against string literal (satisfies &ClientId == &str).
/// ```
/// # use core::str::FromStr;
/// # use ibc_types_core_client::ClientId;
/// let client_id = ClientId::from_str("clientidtwo");
/// assert!(client_id.is_ok());
/// client_id.map(|id| {assert_eq!(&id, "clientidtwo")});
/// ```
impl PartialEq<str> for ClientId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<ClientId> for str {
    fn eq(&self, other: &ClientId) -> bool {
        other.as_str().eq(self)
    }
}
