#!/bin/bash

# release.sh will hopefully allow us to publish all of the necessary crates in
# this repo in the right order. It is assumed that only one person will be
# releasing all crates at the same time.
#
# It has a default set of crates it will publish, which can be overridden by
# way of command line arguments:
#
#   # Release all packages, prompting for each package as to whether to publish
#   ./release.sh
#
#   # Just release the proto and tendermint crates, but nothing else
#   ./release.sh proto tendermint

set -e

# A space-separated list of all the crates we want to publish, in the order in
# which they must be published. It's important to respect this order, since
# each subsequent crate depends on one or more of the preceding ones.
DEFAULT_CRATES="
  ibc-types-domain-type \
  ibc-types-identifier \
  ibc-types-timestamp \
  ibc-types-core-commitment \
  ibc-types-core-client \
  ibc-types-transfer \
  ibc-types-core-connection \
  ibc-types-core-channel \
  ibc-types-lightclients-tendermint \
  ibc-types-path \
  ibc-types"

# Allows us to override the crates we want to publish.
CRATES=${*:-${DEFAULT_CRATES}}

# Additional flags to pass to the "cargo publish" operation for every crate we
# publish.
CARGO_PUBLISH_FLAGS=""

# Allow us to specify a crates.io API token via environment variables. Mostly
# for CI use.
if [ -n "${CRATES_TOKEN}" ]; then
  CARGO_PUBLISH_FLAGS="${CARGO_PUBLISH_FLAGS} --token ${CRATES_TOKEN}"
fi

get_manifest_path() {
  cargo metadata --format-version 1 | jq -r '.packages[]|select(.name == "'"${1}"'")|.manifest_path'
}

get_local_version() {
  cargo metadata --format-version 1 | jq -r '.packages[]|select(.name == "'"${1}"'")|.version'
}

check_version_online() {
  curl -s "https://crates.io/api/v1/crates/${1}" | jq -r 'try .versions[]|select(.num == "'"${2}"'").updated_at'
}

publish() {
  echo "Publishing crate $1..."
  cargo publish --manifest-path "$(get_manifest_path "${1}")" ${CARGO_PUBLISH_FLAGS}
  echo ""
}

dry_publish() {
  echo "Publishing crate $1..."
  cargo publish --dry-run --manifest-path "$(get_manifest_path "${1}")" ${CARGO_PUBLISH_FLAGS}
  echo ""
}

wait_until_available() {
  echo "Waiting for crate ${1} to become available via crates.io..."
  for retry in {1..5}; do
    sleep 5
    ONLINE_DATE="$(check_version_online "${1}" "${2}")"
    if [ -n "${ONLINE_DATE}" ]; then
      echo "Crate ${crate} is now available online"
      break
    else
      if [ "${retry}" == 5 ]; then
        echo "ERROR: Crate should have become available by now"
        exit 1
      else
        echo "Not available just yet. Waiting a few seconds..."
      fi
    fi
  done
  echo "Waiting an additional 10 seconds for crate to propagate through CDN..."
  sleep 10
}

echo "Doing a dry-run"
for crate in ${CRATES}; do
  dry_publish "${crate}"
done

echo "Attempting to publish crate(s): ${CRATES}"

# Since crates.io has rate-limit and we always want to publish 
# every crate in lockstep, we need to wait 5min every 5 pubs
COUNTER=1

for crate in ${CRATES}; do
  VERSION="$(get_local_version "${crate}")"
  ONLINE_DATE="$(check_version_online "${crate}" "${VERSION}")"
  echo "${crate} version number: ${VERSION}"
  if [ -n "${ONLINE_DATE}" ]; then
    echo "${crate} ${VERSION} has already been published at ${ONLINE_DATE}, skipping"
    continue
  fi

  if [ $COUNTER -eq 5 ]; then
    echo "Reached 5 crates, waiting for 5 minutes..."
    sleep 300 # Wait for 5 minutes
    COUNTER=1 # Reset the counter
  fi

  publish "${crate}"
  wait_until_available "${crate}" "${VERSION}"
  ((COUNTER++))
done


echo "Attempting to publish crate(s): ${CRATES}"

# Since crates.io has rate-limit and we always want to publish 
# every crate in lockstep, we need to wait 5min every 5 pubs
COUNTER=1

for crate in ${CRATES}; do
  VERSION="$(get_local_version "${crate}")"
  ONLINE_DATE="$(check_version_online "${crate}" "${VERSION}")"
  echo "${crate} version number: ${VERSION}"
  if [ -n "${ONLINE_DATE}" ]; then
    echo "${crate} ${VERSION} has already been published at ${ONLINE_DATE}, skipping"
    continue
  fi

  if [ $COUNTER -eq 5 ]; then
    echo "Reached 5 crates, waiting for 5 minutes..."
    sleep 300 # Wait for 5 minutes
    COUNTER=1 # Reset the counter
  fi

  publish "${crate}"
  wait_until_available "${crate}" "${VERSION}"
  ((COUNTER++))
done
