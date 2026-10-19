.PHONY: ci-prepare

# Called inside one native package by the SHA-pinned StellarExpert builder.
# Keep SDK 21 on the compiler/target already exercised by the native test CI.
ci-prepare:
	@case "$$(basename "$$PWD")" in campaign_escrow|contributor_pool|reputation_anchor) ;; *) echo "Unexpected native package directory" >&2; exit 1 ;; esac
	rustup toolchain install 1.81.0 --profile minimal --target wasm32-unknown-unknown --no-self-update
	rustup override set 1.81.0
	CARGO_BUILD_JOBS=2 cargo +1.81.0 test --locked
	cargo +1.81.0 fetch --locked --target wasm32-unknown-unknown
	git diff --exit-code -- Cargo.lock Cargo.toml src
