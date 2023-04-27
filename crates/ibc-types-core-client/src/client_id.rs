use core::{
    fmt::{Debug, Display, Error as FmtError, Formatter},
    str::FromStr,
};

use derive_more::Into;

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
    pub fn new(client_type: ClientType, counter: u64) -> Result<Self, ClientIdParseError> {
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
    type Err = ClientIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_client_identifier(s).map(|_| Self(s.to_string()))
    }
}

impl TryFrom<String> for ClientId {
    type Error = ClientIdParseError;

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
/// use core::str::FromStr;
/// use ibc_types::core::ics24_host::identifier::ClientId;
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

use displaydoc::Display;

/// An error while parsing a [`ClientId`].
#[derive(Debug, Display)]
pub enum ClientIdParseError {
    /// identifier `{id}` cannot contain separator '/'
    ContainSeparator { id: String },
    /// identifier `{id}` has invalid length `{length}` must be between `{min}`-`{max}` characters
    InvalidLength {
        id: String,
        length: usize,
        min: usize,
        max: usize,
    },
    /// identifier `{id}` must only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`
    InvalidCharacter { id: String },
    /// identifier cannot be empty
    Empty,
}

#[cfg(feature = "std")]
impl std::error::Error for ClientIdParseError {}

// TODO: should validate_identifier be put in a common crate with timestamps or something

/// Path separator (ie. forward slash '/')
const PATH_SEPARATOR: char = '/';
const VALID_SPECIAL_CHARS: &str = "._+-#[]<>";

/// Default validator function for identifiers.
///
/// A valid identifier only contain lowercase alphabetic characters, and be of a given min and max
/// length.
fn validate_identifier(id: &str, min: usize, max: usize) -> Result<(), ClientIdParseError> {
    assert!(max >= min);

    // Check identifier is not empty
    if id.is_empty() {
        return Err(ClientIdParseError::Empty);
    }

    // Check identifier does not contain path separators
    if id.contains(PATH_SEPARATOR) {
        return Err(ClientIdParseError::ContainSeparator { id: id.into() });
    }

    // Check identifier length is between given min/max
    if id.len() < min || id.len() > max {
        return Err(ClientIdParseError::InvalidLength {
            id: id.into(),
            length: id.len(),
            min,
            max,
        });
    }

    // Check that the identifier comprises only valid characters:
    // - Alphanumeric
    // - `.`, `_`, `+`, `-`, `#`
    // - `[`, `]`, `<`, `>`
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || VALID_SPECIAL_CHARS.contains(c))
    {
        return Err(ClientIdParseError::InvalidCharacter { id: id.into() });
    }

    // All good!
    Ok(())
}

/// Default validator function for Client identifiers.
///
/// A valid identifier must be between 9-64 characters and only contain lowercase
/// alphabetic characters,
fn validate_client_identifier(id: &str) -> Result<(), ClientIdParseError> {
    validate_identifier(id, 9, 64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn parse_invalid_client_id_min() {
        // invalid min client id
        let id = validate_client_identifier("client");
        assert!(id.is_err())
    }

    #[test]
    fn parse_client_id_max() {
        // invalid max client id (test string length is 65)
        let id = validate_client_identifier(
            "f0isrs5enif9e4td3r2jcbxoevhz6u1fthn4aforq7ams52jn5m48eiesfht9ckpn",
        );
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_id_chars() {
        // invalid id chars
        let id = validate_identifier("channel@01", 1, 10);
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_id_empty() {
        // invalid id empty
        let id = validate_identifier("", 1, 10);
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_id_path_separator() {
        // invalid id with path separator
        let id = validate_identifier("id/1", 1, 10);
        assert!(id.is_err())
    }
}
