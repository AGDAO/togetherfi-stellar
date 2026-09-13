#!/usr/bin/env bash

# Compile and run only the LayerZero suites with no pre-existing Hardhat
# artifacts or cache. Existing generated output is moved aside and restored so
# this check does not leave tracked or stale build noise in the worktree.
set -Eeuo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
saved_dir="$(mktemp -d "${TMPDIR:-/tmp}/mofo-layerzero-build.XXXXXX")"
exit_code=0

restore_generated_output() {
  exit_code=$?
  set +e

  rm -rf "$repo_dir/artifacts" "$repo_dir/cache"
  if [[ -e "$saved_dir/artifacts" ]]; then
    mv "$saved_dir/artifacts" "$repo_dir/artifacts"
  fi
  if [[ -e "$saved_dir/cache" ]]; then
    mv "$saved_dir/cache" "$repo_dir/cache"
  fi
  rm -rf "$saved_dir"

  exit "$exit_code"
}
trap restore_generated_output EXIT

cd "$repo_dir"
if [[ -e artifacts ]]; then
  mv artifacts "$saved_dir/artifacts"
fi
if [[ -e cache ]]; then
  mv cache "$saved_dir/cache"
fi

npx hardhat compile
npx hardhat test \
  test/ArbStellarReceiptReceiver.test.js \
  test/ArbStellarSponsorFunding.test.js
