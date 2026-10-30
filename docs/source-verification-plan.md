# Queued explorer source-verification acceptance

The owner requested this work. A passing build or a matching local hash alone
is not StellarExpert source verification.

1. Freeze the exact source reviewed by Zyad and Mohammed Idmoussi, including
   initializer hardening, native pool integration and dependency dispositions.
2. Build the three native packages using their locked dependencies and the
   documented Rust 1.81.0 / wasm32-unknown-unknown baseline. The current upstream
   reusable workflow must not silently replace this with an incompatible
   compiler or change SDK dependencies.
3. Generate GitHub build attestations tied to that commit and publish the
   actual WASM artifacts with source/build inputs and SHA-256 checksums.
   The official [StellarExpert build workflow](https://github.com/stellar-expert/soroban-build-workflow)
   documents the attestation and release approach.
4. Compare the attested artifacts with RPC-retrieved bytes. Historical local
   deployment builds and CI builds previously differed: never claim those
   earlier deployed contracts match a new attestation without a byte comparison.
5. If new deployment is required, deploy separate, authorized testnet instances
   from the exact attested artifacts and preserve earlier IDs and refund evidence.
   Coordinate this with the queued Circle USDC validation rather than deploying
   unrelated duplicate fixtures.
6. Confirm StellarExpert actually indexes and verifies each target contract's
   source; retain its public verification links and the on-chain WASM hashes.
7. Update current contract IDs consistently in manifests, README, the public
   architecture page and application configuration. Separate old historical
   evidence from the new deployed source.

The work is not complete until explorer status and deployed bytes are verified.
This queue does not authorize mainnet, real funds, collecting member seeds,
rewriting history or treating reviewer approval as a transaction signature.
