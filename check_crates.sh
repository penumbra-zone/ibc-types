#!/bin/bash

members=(
    "crates/ibc-types-domain-type"
    "crates/ibc-types-core-client"
    "crates/ibc-types-core-channel"
    "crates/ibc-types-timestamp"
    "crates/ibc-types-identifier"
    "crates/ibc-types-core-connection"
    "crates/ibc-types-core-commitment"
    "crates/ibc-types-lightclients-tendermint"
    "crates/ibc-types-path"
    "crates/ibc-types-transfer"
    "crates/ibc-types"
)

for crate_path in "${members[@]}"; do
  echo "processing $crate_path"
  pushd $crate_path
  cargo check || exit 1
  popd
done

