#![no_std]
//! Data types for the InterBlockchain Communication (IBC) protocol in Rust.
//!
//! This crate will eventually replace the `ibc-types` crate and be renamed `ibc-types`.

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

#[doc(inline)]
pub use ibc_types_domain_type::{DomainType, TypeUrl};

// TODO: anywhere better to put this?
// we don't need/want the whole crate since it should be encapsulated
// in the identifier types themselves
#[doc(inline)]
pub use ibc_types_identifier::IdentifierError;

/// Core IBC data modeling such as clients, connections, and channels.
pub mod core {
    #[doc(inline)]
    pub use ibc_types_core_channel as channel;
    #[doc(inline)]
    pub use ibc_types_core_client as client;
    #[doc(inline)]
    pub use ibc_types_core_commitment as commitment;
    #[doc(inline)]
    pub use ibc_types_core_connection as connection;
}

#[doc(inline)]
pub use ibc_types_timestamp as timestamp;

/// Specific IBC light clients, such as the Tendermint light client.
pub mod lightclients {
    // TODO: add Tendermint light client crate
    #[doc(inline)]
    pub use ibc_types_lightclients_tendermint as tendermint;
}
