use alloc::string::ToString;
use core::fmt::{Display, Error as FmtError, Formatter};

use ibc_proto::{google::protobuf::Any, ibc::mock::Header as RawMockHeader, Protobuf};
use ibc_types_timestamp::Timestamp;

use crate::{error::Error, Height};

pub const MOCK_HEADER_TYPE_URL: &str = "/ibc.mock.Header";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MockHeader {
    pub height: Height,
    pub timestamp: Timestamp,
}

impl Default for MockHeader {
    fn default() -> Self {
        Self {
            height: Height::new(0, 1).unwrap(),
            timestamp: Default::default(),
        }
    }
}

impl Display for MockHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "MockHeader {{ height: {}, timestamp: {} }}",
            self.height, self.timestamp
        )
    }
}

impl Protobuf<RawMockHeader> for MockHeader {}

impl TryFrom<RawMockHeader> for MockHeader {
    type Error = Error;

    fn try_from(raw: RawMockHeader) -> Result<Self, Self::Error> {
        Ok(MockHeader {
            height: raw
                .height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(Error::MissingRawHeader)?,

            timestamp: Timestamp::from_nanoseconds(raw.timestamp)
                .map_err(Error::InvalidPacketTimestamp)?,
        })
    }
}

impl From<MockHeader> for RawMockHeader {
    fn from(value: MockHeader) -> Self {
        RawMockHeader {
            height: Some(value.height.into()),
            timestamp: value.timestamp.nanoseconds(),
        }
    }
}

impl MockHeader {
    pub fn height(&self) -> Height {
        self.height
    }

    pub fn new(height: Height) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(any(test, feature = "std"))]  {
                Self {
                    height,
                    timestamp: Timestamp::now(),
                }
            } else {
                Self {
                    height,
                    timestamp: Timestamp::none(),
                }
            }
        }
    }

    pub fn with_timestamp(self, timestamp: Timestamp) -> Self {
        Self { timestamp, ..self }
    }
}

impl Protobuf<Any> for MockHeader {}

impl TryFrom<Any> for MockHeader {
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            MOCK_HEADER_TYPE_URL => Ok(Protobuf::<RawMockHeader>::decode_vec(&raw.value)
                .map_err(Error::InvalidRawHeader)?),
            _ => Err(Error::UnknownHeaderType {
                header_type: raw.type_url,
            }),
        }
    }
}

impl From<MockHeader> for Any {
    fn from(header: MockHeader) -> Self {
        Any {
            type_url: MOCK_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<RawMockHeader>::encode_vec(header),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ibc_proto::Protobuf;

    #[test]
    fn encode_any() {
        let header = MockHeader::new(Height::new(1, 10).unwrap()).with_timestamp(Timestamp::none());
        let bytes = <MockHeader as Protobuf<Any>>::encode_vec(header);

        assert_eq!(
            &bytes,
            &[
                10, 16, 47, 105, 98, 99, 46, 109, 111, 99, 107, 46, 72, 101, 97, 100, 101, 114, 18,
                6, 10, 4, 8, 1, 16, 10
            ]
        );
    }
}
