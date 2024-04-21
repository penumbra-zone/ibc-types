//! Provides a marker type capturing the relationship between a domain type and a protobuf type.
#![no_std]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

#[cfg(all(feature = "anyhow", feature = "eyre"))]
compile_error!("feature \"anyhow\" and feature \"eyre\" cannot be enabled at the same time");

#[cfg(feature = "anyhow")]
extern crate anyhow;
#[cfg(feature = "eyre")]
extern crate eyre as anyhow;

mod prelude;
use prelude::*;

/// A marker type that captures the relationships between a domain type (`Self`) and a protobuf type (`Self::Proto`).
pub trait DomainType
where
    Self: Clone + Sized + TryFrom<Self::Proto>,
    Self::Proto: prost::Message + prost::Name + Default + From<Self> + Send + Sync + 'static,
    <Self as TryFrom<Self::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
{
    type Proto;

    /// Encode this domain type to a byte vector, via proto type `P`.
    fn encode_to_vec(&self) -> Vec<u8> {
        use prost::Message;
        self.to_proto().encode_to_vec()
    }

    /// Convert this domain type to the associated proto type.
    ///
    /// This uses the `From` impl internally, so it works exactly
    /// like `.into()`, but does not require type inference.
    fn to_proto(&self) -> Self::Proto {
        Self::Proto::from(self.clone())
    }

    /// Decode this domain type from a byte buffer, via proto type `P`.
    fn decode<B: bytes::Buf>(buf: B) -> Result<Self, anyhow::Error> {
        <Self::Proto as prost::Message>::decode(buf)
            .map_err(anyhow::Error::msg)?
            .try_into()
            .map_err(Into::into)
    }
}
