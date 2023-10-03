//! IBC channel-related types.
#![no_std]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod channel;
pub mod packet;

mod commitment;
mod error;
mod identifier;
mod prelude;
mod timeout;
mod version;

pub use channel::{ChannelEnd, Counterparty, IdentifiedChannelEnd};
pub use commitment::{AcknowledgementCommitment, PacketCommitment};
pub use error::{ChannelError, PacketError};
pub use identifier::{ChannelId, PortId};
pub use packet::Packet;
pub use timeout::TimeoutHeight;
pub use version::Version;

pub mod events;
pub mod msgs;

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mocks;
