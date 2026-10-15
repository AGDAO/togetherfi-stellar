#!/usr/bin/env bash
# Works both in the standalone public repository and in the application subtree.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE="${1:?contract package required}"
RUST="${2:?exact Rust version required}"
TARGET="${3:?WASM target required}"
case "$PACKAGE:$RUST:$TARGET" in
  campaign_escrow:1.81.0:wasm32-unknown-unknown|contributor_pool:1.81.0:wasm32-unknown-unknown|reputation_anchor:1.81.0:wasm32-unknown-unknown|layerzero_receipt_adapter:1.90.0:wasm32v1-none|layerzero_funding_inbox:1.90.0:wasm32v1-none) ;;
  *) echo "Unsupported package/toolchain/target tuple" >&2; exit 1 ;;
esac
OUT="$ROOT/evidence/current/$PACKAGE"
mkdir -p "$OUT"
: > "$OUT/WASM.sha256"
cd "$ROOT/contracts/$PACKAGE"
{
  echo "Local/CI source verification only; no deployment or on-chain quorum proof."
  echo "generated_at_utc=$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
  echo "package=$PACKAGE"
  echo "toolchain=$RUST"
  echo "target=$TARGET"
  echo "source_revision=$(git -C "$ROOT" rev-parse HEAD)"
  if git -C "$ROOT" diff --quiet HEAD -- "$ROOT"; then
    echo "tracked_worktree=clean"
  else
    echo "tracked_worktree=modified; file hashes below identify tested inputs"
  fi
  rustup run "$RUST" rustc --version
  rustup run "$RUST" cargo --version
  sha256sum Cargo.toml Cargo.lock
  find src -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
  echo '$ cargo test --locked'
  rustup run "$RUST" cargo test --locked
  echo "\$ cargo build --locked --release --target $TARGET"
  rustup run "$RUST" cargo build --locked --release --target "$TARGET"
  find "target/$TARGET/release" -maxdepth 1 -name '*.wasm' -type f -exec sha256sum {} \; > "$OUT/WASM.sha256"
  test -s "$OUT/WASM.sha256"
  cat "$OUT/WASM.sha256"
} 2>&1 | tee "$OUT/verification.txt"