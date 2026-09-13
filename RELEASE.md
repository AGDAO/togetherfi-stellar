# TogetherFi Stellar and LayerZero public source release

## Release scope

This repository is the public source release for TogetherFi's current, undeployed Stellar grant implementation. It contains:

- Soroban Campaign Escrow v2, Contributor Pool, and Reputation Anchor source and tests;
- Soroban LayerZero Funding Inbox and financially inert receipt adapter source, tests, pinned toolchains, and vendored upstream dependencies;
- the reviewed EVM sponsor-funding, OFT-wrapper boundary, and receipt-receiver source, interfaces, tests, and build manifests under `evm-layerzero/`; and
- `docs/stellar-grant-evidence-package.md`, versioned with this release.

It intentionally excludes the private application repository, backend and database code, deployment credentials, private infrastructure, customer data, and unrelated proprietary application code. This source release is not a deployment, audit, production-readiness statement, or evidence of a live cross-chain route.

## Upstream baseline and provenance

The exact public upstream baseline is commit [`6fd30883bf8c305eb8dd8a8a8fabe7226171332b`](https://github.com/AGDAO/togetherfi-stellar/commit/6fd30883bf8c305eb8dd8a8a8fabe7226171332b). `UPSTREAM_COMMIT` records the same value. Current v2, Contributor Pool, Funding Inbox, receipt-adapter, and EVM LayerZero work was developed after that baseline and is released in the immutable commit that contains this file.

Vendored LayerZero provenance and versions are recorded in package lockfiles, Cargo lockfiles, vendored package metadata, and `evm-layerzero/contracts/vendor/layerzero-v2/PROVENANCE.md`. Vendored code remains subject to its upstream notices and licenses.

## License

TogetherFi-authored source in this repository is released under the MIT License in `LICENSE`. Vendored dependencies retain their original licenses and notices.

## Authorship, responsibility, and AI assistance

TogetherFi and its founder are accountable for the architecture, implementation, testing, review, release approval, deployment decisions, maintenance, and operation of this code. Commit attribution records contributors to the source history but does not transfer that responsibility.

AI-assisted development tools, including Replit Agent, were used in the engineering workflow. The founder approves the architecture, reviews delivered code and tests, controls deployment and release decisions, and remains responsible for the resulting system. AI assistance is not presented as independent security review or proof of correctness.

## Verification

Reviewers should use immutable commit URLs rather than `main`. A public release is source provenance only. Deployment claims additionally require reproducible builds, matching WASM or bytecode hashes, network-specific manifests, verified contract identities, and confirmed transaction evidence.
