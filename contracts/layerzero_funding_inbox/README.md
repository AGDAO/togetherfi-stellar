# TogetherFi Stellar LayerZero funding inbox

This is a separate, not-deployed Soroban package for the optional allowlisted
OFT sponsor-funding path. It is intentionally isolated from `campaign_escrow`,
which still owns native campaign funding, completion, refunds, payout splits,
and contributor-pool credits.

## Status

The package is implemented and locally tested, but is not deployed or live. No
real cross-chain transaction evidence is currently claimed. Local tests use
deterministic endpoint/OFT observations and do not demonstrate live bridge
delivery. This README is not an audit or a production-readiness claim.

## Official LayerZero surface

The repository's installed LayerZero packages were inspected before adding
this package. `@layerzerolabs/oapp-stellar-contracts` and its dependency
packages are version **1.2.55**. There is no official Stellar OFT package in
`node_modules` (the package metadata exposes OApp, endpoint, and composer
interfaces only). Consequently this contract uses the official
`endpoint_v2::ILayerZeroComposer` interface and does not invent or vendor an
OFT interface. Its dependency tree is a private copy of the same official
1.2.55 source used by `layerzero_receipt_adapter`.

The compose gate requires all of the following:

* the actual `executor` argument authenticates with `require_auth`; the
  configured Endpoint V2 is used for the official `clear_compose` queue;
* `from` is the configured local OFT contract;
* the official OFT envelope contains the configured Arbitrum source EID and
  exact bytes32 `compose_from` peer;
* both the bridge asset domain and the Stellar escrow/asset domain match
  immutable configuration; and
* the payload is domain-separated and versioned; the source nonce is retained
  as authenticated evidence but is not required to be contiguous because OFT
  nonces are path-scoped,
  unique by GUID/index and campaign/operation.

The incoming message is decoded according to the official OFT compose
envelope: `nonce:u64 | src_eid:u32 | amount_ld:i128 |
compose_from:bytes32 | compose_msg:bytes`, with big-endian integers. TFZF
custom bytes contain only the campaign ID, operation version, Stellar sponsor
address, sponsor reference, minimum destination amount, deadline, asset
domain, and escrow/asset domain. The pinned official LayerZero Stellar
dependency tree contains the OApp/Endpoint composer surface but no official
Stellar OFT package/API. That composer surface exposes no per-GUID credited
amount, token-transfer receipt, or receiver-side pull hook. Therefore the
configured OFT, the authenticated compose executor, the exact source peer, and
the Endpoint compose queue cleared with official `clear_compose` are the
delivery-provenance trust root.
The fungible-token balance is checked only as defense in depth: held
liabilities must remain no greater than the configured token balance. A
receiver cannot distinguish an ambient token transfer from an OFT transfer.
The full wrapped message must be present in the Endpoint's compose queue and
is cleared before the liability/replay records are committed.
Balance deficits, expired instructions, insufficient destination amounts,
malformed or unqueued messages, duplicates, and wrong path/domain abort
atomically. GUID/index and campaign/version identities provide replay
prevention; source nonce gaps and out-of-order execution are valid.

The inbox has an admin-authenticated global pause. While paused, new compose
liabilities and sponsor claims fail. Sponsor cancellation and permissionless
expired release remain available so a pause cannot trap pending sponsor funds.
If the configured OFT is compromised and lies about `amount_ld`, the inbox
cannot detect that lie from the fungible-token balance; operators must pause
the route and replace the immutable inbox/OFT configuration.

## Intent lifecycle and deliberate escrow boundary

Every pending liability is a `FundingIntent` containing campaign ID, operation
version, sponsor address and reference, the authenticated `amount_ld`,
minimum destination amount, deadline, the authenticated envelope nonce and
GUID/index, and escrow/asset domain. The source EID and compose-from peer are
validated against immutable route configuration. Only the exact sponsor
in the authenticated instruction can claim or cancel, and the repeated
sponsor reference must match. After expiry, anyone may release a pending
intent only to that exact committed sponsor; there is no owner confiscation or
recovery address.

Instance configuration/accounting and every persistent intent/replay entry are
kept alive with explicit Soroban TTL extensions on entrypoints, writes, and
reads. If an intent still reaches archival, the committed sponsor must submit
a transaction RestoreFootprint for that intent key and then call
`restore_intent` with the original sponsor reference; this refreshes the entry
so the sponsor can use `claim` or `cancel` (or anyone can call
`release_expired`).

The inbox does **not** invoke `CampaignEscrow`. LayerZero authenticates a
cross-chain message and the OFT path, but it does not establish the Soroban
authorization that `CampaignEscrow::fund` requires, nor can it preserve the
escrow's original-funder refund invariant if an arbitrary bridge payload were
allowed to create a campaign. Automatic funding would therefore weaken the
sponsor-auth and refund guarantees. The safe integration is explicit:

1. the authenticated bridge instruction creates a pending inbox liability;
2. the sponsor authorizes `claim` to receive the locally held tokens; and
3. the sponsor submits a separate native Soroban `CampaignEscrow::fund`
   transaction.

Bridge delivery can never complete or refund a campaign and can never allocate
contributor pools.

This package is an undeployed implementation artifact. The inbox is immutable
for a deployment: changing the token, route, trust boundary, pause policy, or
liability schema requires a replacement deployment and an explicit versioned
migration plan. Existing liabilities must be claimed, cancelled, or
expired-released under the old inbox; administrators cannot sweep or copy the
configured-token balance into a replacement. No deployment or migration is
authorized by this workstream.

**Independent-review gate:** an independent reviewer must assess whether this
explicit configured-OFT trust-boundary treatment closes the delivery-provenance
finding, given that no Soroban receiver can distinguish ambient fungible-token
transfers from official OFT transfers. Deployment and acceptance of real funds
remain blocked until that assessment and any resulting remediation are
complete.

## Isolated toolchain

This package pins Soroban SDK 25.1.1 and Rust 1.90. It has its own lockfile and
vendored official source tree; it is not a member of the SDK 21/Rust 1.81
campaign contract workspace. No deployment or live-fund operation is implied.

```bash
rustup toolchain install 1.90.0
cargo +1.90.0 test
cargo +1.90.0 build --target wasm32-unknown-unknown --release
```