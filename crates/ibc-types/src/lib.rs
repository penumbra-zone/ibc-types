//! This crate defines common data structures for Inter-Blockchain Communication
//! (IBC) messages that can be reused by different IBC implementations or IBC
//! ecosystem tooling.
//!
//! Unlike `ibc-rs`, which provides a specific and opinionated implementation of
//! IBC, `ibc-types` just defines Rust types that allow working with IBC
//! messages, allowing an IBC implementation or IBC ecosystem tooling to be
//! built on top using a common language.
//!
//! In addition to defining Rust types for IBC messages, `ibc-types` also
//! defines Rust types for IBC events, and provides code for parsing IBC events
//! to and from ABCI messages.  IBC events are de facto a critical part of IBC,
//! in that they're needed to interoperate with relayers, but are not really
//! specified anywhere.  Providing event parsing code in `ibc-types` allows IBC
//! implementations and relayer implementations to share common code for
//! producing and consuming events.
//!
//! The `ibc-types` crate is a top-level wrapper crate re-exporting the contents
//! of subcrates scoped by IBC module. For example, the `ibc-types` crate
//! re-exports the client types defined in the `ibc-types-core-client` crate, as
//! well as the types for the Tendermint light client defined in the
//! `ibc-types-lightclients-tendermint` crate.  This structure means that
//! external users of the library can use one catch-all crate, but allows
//! dependency relationships between different IBC modules. For example, the
//! Tendermint light client can depend on the core client types.  This prevents
//! cyclic dependency issues when creating new IBC light clients.
#![no_std]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

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

#[doc(inline)]
pub use ibc_types_path as path;

#[doc(inline)]
pub use ibc_types_transfer as transfer;
