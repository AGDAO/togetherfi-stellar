#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(git -C "$ROOT" rev-parse --show-toplevel)"
OUTPUT_DIR="${1:-$ROOT/evidence/soroban-tests}"
mkdir -p "$OUTPUT_DIR"

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup is required to select the pinned Rust toolchains." >&2
  exit 1
fi

SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD)"
GENERATED_AT="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"

(
  cd "$REPO_ROOT"
  {
    # Use the same explicit public-review boundary in the application subtree
    # and standalone repository. Never sweep unrelated application files.
    node "$ROOT/scripts/review-package.mjs" --paths |
      while IFS= read -r file; do
        if [[ "$ROOT" == "$REPO_ROOT" ]]; then
          printf '%s\n' "$file"
        else
          printf '%s\n' "${ROOT#"$REPO_ROOT"/}/$file"
        fi
      done
  } |
    LC_ALL=C sort -u |
    while IFS= read -r file; do
      printf '%s  %s\n' "$(sha256sum "$file" | awk '{print $1}')" "$file"
    done
) >"$OUTPUT_DIR/INPUTS.sha256"
INPUT_MANIFEST_SHA256="$(sha256sum "$OUTPUT_DIR/INPUTS.sha256" | awk '{print $1}')"

run_package() {
  local package="$1"
  local toolchain="$2"
  local target="$3"
  local wasm_name="$4"
  local package_dir="$ROOT/contracts/$package"
  local transcript="$OUTPUT_DIR/$package.txt"
  local wasm="$package_dir/target/$target/release/$wasm_name"

  (
    echo "TogetherFi Soroban reproducibility transcript"
    echo "generated_at_utc=$GENERATED_AT"
    echo "repository_base_commit=$SOURCE_COMMIT"
    echo "input_manifest_sha256=$INPUT_MANIFEST_SHA256"
    echo "package=$package"
    echo "toolchain=$toolchain"
    echo "target=$target"
    echo "rustc=$(rustup run "$toolchain" rustc --version)"
    echo "cargo=$(rustup run "$toolchain" cargo --version)"
    echo "cargo_toml_sha256=$(sha256sum "$package_dir/Cargo.toml" | awk '{print $1}')"
    echo "cargo_lock_sha256=$(sha256sum "$package_dir/Cargo.lock" | awk '{print $1}')"
    echo "source_tree_sha256=$(
      cd "$package_dir"
      find src -type f -print0 |
        LC_ALL=C sort -z |
        xargs -0 sha256sum |
        sha256sum |
        awk '{print $1}'
    )"
    echo
    echo "\$ rustup run $toolchain cargo test --locked"
    (
      cd "$package_dir"
      rustup run "$toolchain" cargo test --locked
    )
    echo
    echo "\$ rustup run $toolchain cargo build --locked --target $target --release"
    (
      cd "$package_dir"
      rustup run "$toolchain" cargo build --locked --target "$target" --release
    )
    test -f "$wasm"
    echo
    echo "wasm_path=${wasm#"$REPO_ROOT/"}"
    echo "wasm_sha256=$(sha256sum "$wasm" | awk '{print $1}')"
  ) >"$transcript" 2>&1
  sed -i "s|$REPO_ROOT|<repo>|g" "$transcript"
}

run_package campaign_escrow 1.81.0 wasm32-unknown-unknown campaign_escrow.wasm
run_package contributor_pool 1.81.0 wasm32-unknown-unknown contributor_pool.wasm
run_package reputation_anchor 1.81.0 wasm32-unknown-unknown reputation_anchor.wasm

{
  echo "# TogetherFi Soroban Test Evidence"
  echo
  echo "- Generated (UTC): \`$GENERATED_AT\`"
  echo "- Repository base commit: \`$SOURCE_COMMIT\`"
  echo "- Canonical input manifest: \`INPUTS.sha256\`"
  echo "- Input manifest SHA-256: \`$INPUT_MANIFEST_SHA256\`"
  echo "- Status: local reproducibility evidence only; not deployment, testnet, or live-route proof."
  echo
  echo "| Package | Toolchain | Target | Result | WASM SHA-256 |"
  echo "|---|---|---|---|---|"
  for spec in \
    "campaign_escrow|1.81.0|wasm32-unknown-unknown" \
    "contributor_pool|1.81.0|wasm32-unknown-unknown" \
    "reputation_anchor|1.81.0|wasm32-unknown-unknown"; do
    IFS='|' read -r package toolchain target <<<"$spec"
    transcript="$OUTPUT_DIR/$package.txt"
    result="$(grep 'test result:' "$transcript" | tail -1 | sed 's/^[[:space:]]*//')"
    wasm_hash="$(grep '^wasm_sha256=' "$transcript" | cut -d= -f2)"
    echo "| \`$package\` | Rust \`$toolchain\` | \`$target\` | $result | \`$wasm_hash\` |"
  done
  echo
  echo "Each package transcript records the exact command, compiler and Cargo versions,"
  echo "Cargo manifest and lockfile hashes, checkout-independent source-tree hash,"
  echo "test output, and WASM hash. \`INPUTS.sha256\` binds all generator, source,"
  echo "vendored dependency, documentation, and environment-definition inputs."
} >"$OUTPUT_DIR/README.md"

echo "Evidence written to ${OUTPUT_DIR#"$REPO_ROOT/"}"