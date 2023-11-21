use crate::prelude::*;
use core::cmp::Ordering;

use core::{num::ParseIntError, str::FromStr};

use displaydoc::Display;
use ibc_proto::Protobuf;

use ibc_proto::ibc::core::client::v1::Height as RawHeight;

use crate::error::Error;

/// An IBC height, containing a revision number (epoch) and a revision height (block height).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with_serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "RawHeight", into = "RawHeight")
)]
pub struct Height {
    /// Previously known as "epoch"
    pub revision_number: u64,

    /// The height of a block
    pub revision_height: u64,
}

impl Height {
    pub fn new(revision_number: u64, revision_height: u64) -> Result<Self, Error> {
        if revision_height == 0 {
            return Err(Error::InvalidHeight);
        }

        Ok(Self {
            revision_number,
            revision_height,
        })
    }

    pub fn revision_number(&self) -> u64 {
        self.revision_number
    }

    pub fn revision_height(&self) -> u64 {
        self.revision_height
    }

    pub fn add(&self, delta: u64) -> Height {
        Height {
            revision_number: self.revision_number,
            revision_height: self.revision_height + delta,
        }
    }

    pub fn increment(&self) -> Height {
        self.add(1)
    }

    pub fn sub(&self, delta: u64) -> Result<Height, Error> {
        if self.revision_height <= delta {
            return Err(Error::InvalidHeightResult);
        }

        Ok(Height {
            revision_number: self.revision_number,
            revision_height: self.revision_height - delta,
        })
    }

    pub fn decrement(&self) -> Result<Height, Error> {
        self.sub(1)
    }
}

impl PartialOrd for Height {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Height {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.revision_number < other.revision_number {
            Ordering::Less
        } else if self.revision_number > other.revision_number {
            Ordering::Greater
        } else if self.revision_height < other.revision_height {
            Ordering::Less
        } else if self.revision_height > other.revision_height {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Protobuf<RawHeight> for Height {}

impl TryFrom<RawHeight> for Height {
    type Error = Error;

    fn try_from(raw_height: RawHeight) -> Result<Self, Self::Error> {
        Height::new(raw_height.revision_number, raw_height.revision_height)
    }
}

impl From<Height> for RawHeight {
    fn from(ics_height: Height) -> Self {
        RawHeight {
            revision_number: ics_height.revision_number,
            revision_height: ics_height.revision_height,
        }
    }
}

impl core::fmt::Debug for Height {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct("Height")
            .field("revision", &self.revision_number)
            .field("height", &self.revision_height)
            .finish()
    }
}

/// Custom debug output to omit the packet data
impl core::fmt::Display for Height {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}-{}", self.revision_number, self.revision_height)
    }
}

/// An error while parsing a [`Height`].
#[derive(Debug, Display)]
pub enum HeightParseError {
    /// cannot convert into a `Height` type from string `{height}`
    HeightConversion {
        height: String,
        error: ParseIntError,
    },
    /// attempted to parse a height with invalid format (not in the form `revision_number-revision_height`)
    InvalidFormat,
    /// attempted to parse an invalid zero height
    ZeroHeight,
}

#[cfg(feature = "std")]
impl std::error::Error for HeightParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            HeightParseError::HeightConversion { error: e, .. } => Some(e),
            HeightParseError::ZeroHeight => None,
            HeightParseError::InvalidFormat => None,
        }
    }
}

impl TryFrom<&str> for Height {
    type Error = HeightParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split: Vec<&str> = value.split('-').collect();

        if split.len() != 2 {
            return Err(HeightParseError::InvalidFormat);
        }

        let revision_number =
            split[0]
                .parse::<u64>()
                .map_err(|e| HeightParseError::HeightConversion {
                    height: value.to_owned(),
                    error: e,
                })?;
        let revision_height =
            split[1]
                .parse::<u64>()
                .map_err(|e| HeightParseError::HeightConversion {
                    height: value.to_owned(),
                    error: e,
                })?;

        Height::new(revision_number, revision_height).map_err(|_| HeightParseError::ZeroHeight)
    }
}

impl From<Height> for String {
    fn from(height: Height) -> Self {
        format!("{}-{}", height.revision_number, height.revision_height)
    }
}

impl FromStr for Height {
    type Err = HeightParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Height::try_from(s)
    }
}
