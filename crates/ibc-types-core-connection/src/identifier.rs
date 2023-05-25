use core::convert::{From, Infallible};
use core::fmt::{Debug, Display, Error as FmtError, Formatter};
use core::str::FromStr;

use crate::prelude::*;

// TODO: where does C

/// This type is subject to future changes.
///
/// TODO: ChainId validation is not standardized yet.
///       `is_epoch_format` will most likely be replaced by validate_chain_id()-style function.
///       See: <https://github.com/informalsystems/ibc-rs/pull/304#discussion_r503917283>.
///
/// Also, contrast with tendermint-rs `ChainId` type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainId {
    pub id: String,
    pub version: u64,
}

impl ChainId {
    /// Creates a new `ChainId` given a chain name and an epoch number.
    ///
    /// The returned `ChainId` will have the format: `{chain name}-{epoch number}`.
    /// ```
    /// use ibc_types::core::ics24_host::identifier::ChainId;
    ///
    /// let epoch_number = 10;
    /// let id = ChainId::new("chainA".to_string(), epoch_number);
    /// assert_eq!(id.version(), epoch_number);
    /// ```
    pub fn new(name: String, version: u64) -> Self {
        Self {
            id: format!("{name}-{version}"),
            version,
        }
    }

    pub fn from_string(id: &str) -> Self {
        let version = if Self::is_epoch_format(id) {
            Self::chain_version(id)
        } else {
            0
        };

        Self {
            id: id.to_string(),
            version,
        }
    }

    /// Get a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.id
    }

    // TODO: this should probably be named epoch_number.
    /// Extract the version from this chain identifier.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Extract the version from the given chain identifier.
    /// ```
    /// use ibc_types::core::ics24_host::identifier::ChainId;
    ///
    /// assert_eq!(ChainId::chain_version("chain--a-0"), 0);
    /// assert_eq!(ChainId::chain_version("ibc-10"), 10);
    /// assert_eq!(ChainId::chain_version("cosmos-hub-97"), 97);
    /// assert_eq!(ChainId::chain_version("testnet-helloworld-2"), 2);
    /// ```
    pub fn chain_version(chain_id: &str) -> u64 {
        if !ChainId::is_epoch_format(chain_id) {
            return 0;
        }

        let split: Vec<_> = chain_id.split('-').collect();
        split
            .last()
            .expect("get revision number from chain_id")
            .parse()
            .unwrap_or(0)
    }

    /// is_epoch_format() checks if a chain_id is in the format required for parsing epochs
    /// The chainID must be in the form: `{chainID}-{version}`
    /// ```
    /// use ibc_types::core::ics24_host::identifier::ChainId;
    /// assert_eq!(ChainId::is_epoch_format("chainA-0"), false);
    /// assert_eq!(ChainId::is_epoch_format("chainA"), false);
    /// assert_eq!(ChainId::is_epoch_format("chainA-1"), true);
    /// assert_eq!(ChainId::is_epoch_format("c-1"), true);
    /// ```
    pub fn is_epoch_format(chain_id: &str) -> bool {
        let re = safe_regex::regex!(br".*[^-]-[1-9][0-9]*");
        re.is_match(chain_id.as_bytes())
    }

    /// with_version() checks if a chain_id is in the format required for parsing epochs, and if so
    /// replaces it's version with the specified version
    /// ```
    /// use ibc_types::core::ics24_host::identifier::ChainId;
    /// assert_eq!(ChainId::new("chainA".to_string(), 1).with_version(2), ChainId::new("chainA".to_string(), 2));
    /// assert_eq!("chain1".parse::<ChainId>().unwrap().with_version(2), "chain1".parse::<ChainId>().unwrap());
    /// ```
    pub fn with_version(mut self, version: u64) -> Self {
        if Self::is_epoch_format(&self.id) {
            self.id = {
                let mut split: Vec<&str> = self.id.split('-').collect();
                let version = version.to_string();
                *split.last_mut().unwrap() = &version;
                split.join("-")
            };
            self.version = version;
        }
        self
    }
}

impl FromStr for ChainId {
    type Err = Infallible;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string(id))
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.id)
    }
}

impl From<ChainId> for tendermint::chain::Id {
    fn from(id: ChainId) -> Self {
        tendermint::chain::Id::from_str(id.as_str()).unwrap()
    }
}

impl From<tendermint::chain::Id> for ChainId {
    fn from(id: tendermint::chain::Id) -> Self {
        ChainId::from_str(id.as_str()).unwrap()
    }
}

impl Default for ChainId {
    fn default() -> Self {
        "defaultChainId".to_string().parse().unwrap()
    }
}

impl From<String> for ChainId {
    fn from(value: String) -> Self {
        Self::from_string(&value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConnectionId(pub String);

impl ConnectionId {
    /// Builds a new connection identifier. Connection identifiers are deterministically formed from
    /// two elements: a prefix `prefix`, and a monotonically increasing `counter`; these are
    /// separated by a dash "-". The prefix is currently determined statically (see
    /// `ConnectionId::prefix()`) so this method accepts a single argument, the `counter`.
    ///
    /// ```
    /// # use ibc_types::core::ics24_host::identifier::ConnectionId;
    /// let conn_id = ConnectionId::new(11);
    /// assert_eq!(&conn_id, "connection-11");
    /// ```
    pub fn new(identifier: u64) -> Self {
        let id = format!("{}-{}", Self::prefix(), identifier);
        Self::from_str(id.as_str()).unwrap()
    }

    /// Returns the static prefix to be used across all connection identifiers.
    pub fn prefix() -> &'static str {
        "connection"
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// This implementation provides a `to_string` method.
impl Display for ConnectionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ConnectionId {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_connection_identifier(s).map(|_| Self(s.to_string()))
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Equality check against string literal (satisfies &ConnectionId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc_types::core::ics24_host::identifier::ConnectionId;
/// let conn_id = ConnectionId::from_str("connectionId-0");
/// assert!(conn_id.is_ok());
/// conn_id.map(|id| {assert_eq!(&id, "connectionId-0")});
/// ```
impl PartialEq<str> for ConnectionId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

use displaydoc::Display;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum ValidationError {
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
    /// Invalid channel id in counterparty
    InvalidCounterpartyChannelId,
}

#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}

/// Path separator (ie. forward slash '/')
const PATH_SEPARATOR: char = '/';
const VALID_SPECIAL_CHARS: &str = "._+-#[]<>";

/// Default validator function for identifiers.
///
/// A valid identifier only contain lowercase alphabetic characters, and be of a given min and max
/// length.
pub fn validate_identifier(id: &str, min: usize, max: usize) -> Result<(), ValidationError> {
    assert!(max >= min);

    // Check identifier is not empty
    if id.is_empty() {
        return Err(ValidationError::Empty);
    }

    // Check identifier does not contain path separators
    if id.contains(PATH_SEPARATOR) {
        return Err(ValidationError::ContainSeparator { id: id.into() });
    }

    // Check identifier length is between given min/max
    if id.len() < min || id.len() > max {
        return Err(ValidationError::InvalidLength {
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
        return Err(ValidationError::InvalidCharacter { id: id.into() });
    }

    // All good!
    Ok(())
}

/// Default validator function for Client identifiers.
///
/// A valid identifier must be between 9-64 characters and only contain lowercase
/// alphabetic characters,
pub fn validate_client_identifier(id: &str) -> Result<(), ValidationError> {
    validate_identifier(id, 9, 64)
}

/// Default validator function for Connection identifiers.
///
/// A valid Identifier must be between 10-64 characters and only contain lowercase
/// alphabetic characters,
pub fn validate_connection_identifier(id: &str) -> Result<(), ValidationError> {
    validate_identifier(id, 10, 64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn parse_invalid_connection_id_min() {
        // invalid min connection id
        let id = validate_connection_identifier("connect01");
        assert!(id.is_err())
    }

    #[test]
    fn parse_connection_id_max() {
        // invalid max connection id (test string length is 65)
        let id = validate_connection_identifier(
            "ihhankr30iy4nna65hjl2wjod7182io1t2s7u3ip3wqtbbn1sl0rgcntqc540r36r",
        );
        assert!(id.is_err())
    }

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
