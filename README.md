# ibc-types

This crate defines common data structures for Inter-Blockchain Communication
(IBC) messages that can be reused by different IBC implementations or IBC
ecosystem tooling.

Unlike [ibc-rs], which provides a specific and opinionated implementation of
IBC, `ibc-types` just defines Rust types that allow working with IBC messages,
allowing an IBC implementation or IBC ecosystem tooling to be built on top using
a common language.

In addition to defining Rust types for IBC messages, `ibc-types` also defines
Rust types for IBC events, and provides code for parsing IBC events to and from
ABCI messages.  IBC events are de facto a critical part of IBC, in that they're
needed to interoperate with relayers, but are not really specified anywhere.
Providing event parsing code in `ibc-types` allows IBC implementations and
relayer implementations to share common code for producing and consuming events.

The `ibc-types` crate is a top-level wrapper crate re-exporting the contents
of subcrates scoped by IBC module. For example, the `ibc-types` crate
re-exports the client types defined in the `ibc-types-core-client` crate, as
well as the types for the Tendermint light client defined in the
`ibc-types-lightclients-tendermint` crate.  This structure means that
external users of the library can use one catch-all crate, but allows
dependency relationships between different IBC modules. For example, the
Tendermint light client can depend on the core client types.  This prevents
cyclic dependency issues when creating new IBC light clients.

## Contributing

IBC is specified in English in the [cosmos/ibc repo][ibc]. Any
protocol changes or clarifications should be contributed there.

This repo contains Rust datatypes modeling IBC messages.  

## Versioning

We follow [Semantic Versioning][semver], though APIs are still
under active development.

## Resources

- [IBC Website][ibc-homepage]
- [IBC Specification][ibc]
- [IBC Go implementation][ibc-go]

## License

Copyright © 2023 ibc-types authors.

This crate was originally forked from ibc-rs:

Copyright © 2022 Informal Systems Inc. and ibc-rs authors.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use the files in this repository except in compliance with the License. You may
obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

[//]: # (badges)
[docs-image]: https://docs.rs/ibc/badge.svg
[docs-link]: https://docs.rs/ibc/
[build-image]: https://github.com/cosmos/ibc-rs/workflows/Rust/badge.svg
[build-link]: https://github.com/cosmos/ibc-rs/actions?query=workflow%3ARust
[codecov-image]: https://codecov.io/gh/cosmos/ibc-rs/branch/main/graph/badge.svg?token=wUm2aLCOu
[codecov-link]: https://codecov.io/gh/cosmos/ibc-rs
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/cosmos/ibc-rs/blob/main/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg
[rustc-version]: https://img.shields.io/badge/rustc-1.60+-blue.svg

[//]: # (general links)
[ibc-rs]: https://github.com/cosmos/ibc-rs
[ibc]: https://github.com/cosmos/ibc
[ibc-go]: https://github.com/cosmos/ibc-go
[ibc-homepage]: https://cosmos.network/ibc
[cosmos-link]: https://cosmos.network
[semver]: https://semver.org/
