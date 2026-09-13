# TogetherFi × Stellar — Soroban Contracts

## Grant preparation status

The Campaign Escrow v2 source in this working tree is implemented and locally
tested but **has not been deployed**. No deployment is authorized as part of
this grant-preparation work. The contract IDs and transactions below document
the earlier v1 testnet baseline only; they are existing evidence and must not be
claimed as new grant-funded delivery.

This release is derived from upstream baseline commit
`6fd30883bf8c305eb8dd8a8a8fabe7226171332b`, recorded in `UPSTREAM_COMMIT`.
See `RELEASE.md` for release scope, attribution, responsibility, and AI-assistance disclosure.

[![Tests](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml/badge.svg)](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml)

Two Soroban smart contracts deployed on Stellar testnet as part of the
[TogetherFi](https://togetherfi.io) Stellar Community Fund grant application.

---

## Existing v1 deployed contracts (Stellar testnet, historical)

The addresses and transactions below are **v1 historical evidence only**.
They are not the v2 implementation and do not describe a new deployment.

| Contract | Address |
|---|---|
| Campaign Escrow | `CDEAOPTIFKCOFOOHNXXFLF3WAUSTXZKMIXN7NM5OM2BO3CLBX2AND34N` |
| Reputation Anchor | `CBJVDSS6VUWNORYG2W7ZMHPGMCBXQN5A2CKK5OSS7L2WFR6RSFLGQILC` |

---

## Campaign Escrow v2 (`contracts/campaign_escrow`) — not deployed

Holds USDC for a TogetherFi creator campaign and releases it atomically on
completion with the platform's fixed payout split:

| Recipient | Share |
|---|---|
| Creator wallet | 82% |
| Platform treasury | 10% |
| MOFO contributor pool | 5% |
| Community contributor pool | 2% |
| Verified referrer wallet | 1% (mandatory; activation fails without one) |

### Functions

| Function | Caller | Action |
|---|---|---|
| `initialize(governance, settlement, token, service_treasury, nft_pool, revenue_pool)` | governance | One-time setup |
| `fund(campaign_id, sponsor, amount)` | sponsor signs | Transfer USDC sponsor → contract |
| `activate(campaign_id, creator, referrer, terms_hash)` | stored sponsor | Commit payout destinations and terms |
| `complete(campaign_id, operation_version)` | settlement authority | Release with split atomically |
| `refund_expired(campaign_id)` | anyone after expiry | Return full balance to stored sponsor |
| `get_campaign(id)` | anyone | Read campaign state |

The two pool addresses are fixed at escrow initialization. They must be
dedicated `Contributor Pool` deployments for the same token. Only the fixed
5% and 2% campaign allocations may enter those pools through authenticated,
one-time campaign credits. Every campaign requires a verified referrer; there
is no fallback recipient for the `1%` referrer leg.

Campaign funding is at least 100 base units (`0.0000100` for a 7-decimal
asset), which ensures every fixed 5% / 2% / 1% payout leg is nonzero.

## Contributor Pool (`contracts/contributor_pool`) — not deployed

Deploy **two separately initialized instances**: one for the 5% MOFO
contributor allocation and one for the 2% community contributor allocation.
An escrow completion transfers its respective pool allocation to that
instance. The pool contract cannot choose a recipient or sweep money to an
operator:

- `publish_manifest(...)` requires both governance and an independent
  eligibility authority to authorize the same transaction.
- The manifest stores an immutable policy hash, manifest hash, exact
  recipients, exact base-unit amounts, and claim expiry. One manifest is
  capped at 64 recipients; publish separate cycles rather than truncating a
  candidate set.
- Each recipient must authorize their own `claim(manifest_id,index)`.
- An allocation cannot be claimed twice. Claims are bounded by the actual
  token balance and all outstanding manifest liabilities.
- A direct token transfer is never allocatable reward funding. The configured
  Campaign Escrow must authenticate each campaign's one-time credit after its
  matching 5% or 2% transfer, and the escrow refuses pool addresses whose
  configured escrow or token do not match.
- After expiry, `expire_manifest` releases only the unclaimed reservation to
  the pool's carry-forward balance; it makes no transfer to governance.
- Governance and eligibility authority must always be distinct addresses;
  neither can publish a recipient manifest alone.
- Claim expiry is limited to 30 days. Refreshing a manifest's TTL refreshes
  every allocation as well, and a scheduled TTL touch is required while a
  claim window remains active.

Off-chain eligibility must be revalidated from auditable evidence before the
two authorities publish: MOFO recipients need verified ownership and
campaign-promotion evidence; community recipients need verified qualifying
content and all required UTC check-ins. Missing or ambiguous evidence fails
closed. The canonical manifest serialization and SHA-256 policy/manifest
hashes are documented by the application and must be independently reviewed
before any deployment.

### Existing v1 testnet transactions

| Step | Transaction hash |
|---|---|
| Deploy | `54b6a37c3b95c86135aa8ae369f761f99a4204125bc025363bf2d3171101f553` |
| Initialize | `ff9e9c6347fd4acb86986dfd52e87a649b87b350338a096bcbb6e30c5b05b1bd` |
| Fund (end-to-end test) | `ebced8761f76e5e1bbdb5d0fe0859f095ce999e2fb5487358474399484d25c89` |
| Complete (end-to-end test) | `268ad813ee9192a9a6838b03173e3403d83df7bf1ab3cf17ae2c8790f574496b` |

Split verified on-chain: 82.5% creator / 10% platform treasury / 5% revenue pool / 2.5% referrer — escrow drained to zero.

> Note: XLM native asset used as USDC stand-in for the testnet test.
> Mainnet deployment will use the official Stellar USDC token contract on grant approval.

---

## Reputation Anchor (`contracts/reputation_anchor`)

Stores creator TogetherScores permanently on Stellar as the authoritative
on-chain identity record. The historical contract schema names one optional
external reference field `arbitrum_tx_hash`. That field is application metadata,
not authenticated evidence for either the financially inert receipt channel or
the allowlisted OFT sponsor-funding route.

### Functions

| Function | Caller | Action |
|---|---|---|
| `initialize(admin)` | admin | One-time setup |
| `anchor_score(profile_id, creator, score, tier, arbitrum_tx_hash)` | admin only | Write/update score |
| `get_score(profile_id)` | anyone | Read one score record |
| `get_scores_batch(profile_ids)` | anyone | Batch read (missing entries skipped) |

### ScoreRecord fields

```rust
pub struct ScoreRecord {
    pub profile_id:       u64,    // TogetherFi DB profile ID
    pub creator:          Address, // Stellar G-key
    pub score:            u32,    // 0–1000 (TogetherScore v2)
    pub tier:             Symbol, // "bronze" | "silver" | "gold"
    pub arbitrum_tx_hash: String, // historical optional external reference
    pub anchored_at:      u64,   // unix timestamp (ledger time)
}
```

### ScoreAnchored event

Emitted by `anchor_score` on every call. Indexed by `(profile_id, score, tier, anchored_at, arbitrum_tx_hash)`. Any Stellar explorer can verify the anchor on-chain.

## LayerZero Receipt Adapter (`contracts/layerzero_receipt_adapter`) — not deployed

The LayerZero V2 adapter is an optional informational messaging component. It
publishes versioned, domain-separated receipts only after a Stellar campaign has
already reached a final native state. The Soroban Campaign Escrow remains the
sole authority for funding, completion, refunds, payout splits, and contributor
credits.

- Receipt messages are financially inert: they carry no campaign principal and
  are not the OFT sponsor-funding route.
- A send or delivery failure cannot change a successful Stellar settlement.
- Inbound messages cannot call the escrow, release or refund funds, select
  recipients, or allocate contributor balances.
- The configured Endpoint V2 and exact remote peer are authenticated.
- Message GUIDs and campaign operation identities are rejected when replayed or
  duplicated; any inbound receipt nonce check is scoped to its source path.
- The peer receiver on Arbitrum stores receipt metadata only and has no
  value-moving methods.

The adapter is isolated from the existing Soroban SDK 21 contracts because the
pinned official LayerZero Stellar OApp package uses Soroban SDK 25.1.1 and Rust
1.90.0. This avoids weakening or silently upgrading the deployment-compatible
Rust 1.81 build boundary for Campaign Escrow, Contributor Pool, and Reputation
Anchor. No LayerZero contract ID, Endpoint address, peer address, or live
delivery is claimed in this repository. The adapter is implemented and locally
tested, but is not deployed or live; no real cross-chain transaction evidence
is currently claimed.

The separate `layerzero_funding_inbox` package implements the optional
allowlisted OFT sponsor-funding route. OFT delivery goes into that inbox as a
pending liability, not campaign funds. The exact sponsor must claim the
liability and then submit a separate native `CampaignEscrow::fund` transaction.
Neither OFT delivery nor a financially inert receipt invokes the escrow.

The receipt adapter and funding inbox are implementation artifacts only. They
are not an audit or a production-readiness claim, and no real cross-chain
transaction evidence is presented here.

## LayerZero Funding Inbox (`contracts/layerzero_funding_inbox`) — not deployed

The optional bridge funding path is a separate Soroban package. It uses the
official LayerZero Stellar 1.2.55 `ILayerZeroComposer` interface, authenticates
the actual compose executor, and accepts only the configured local OFT, exact
Arbitrum source EID/peer from the official OFT envelope, configured
bridge-asset domain, and configured Stellar escrow/asset domain. It stores
versioned funding intents and binds delivery provenance to the pinned official
configured OFT, authenticated compose executor, exact source peer, and
Endpoint queue cleared through official `clear_compose`. The pinned official
LayerZero Stellar dependency tree contains the OApp/Endpoint composer surface
but no official Stellar OFT package/API or per-GUID credited amount or
receiver-side transfer hook, so the configured token balance is only a
defense-in-depth check that held liabilities do not exceed balance; ambient transfers are
indistinguishable from OFT transfers to the receiver. A compromised OFT can
lie about `amount_ld`, which requires route pause and immutable inbox/OFT
replacement. GUID/index duplicates, campaign operation duplicates,
malformed or unqueued messages, wrong-domain messages, unsupported assets,
expired instructions, insufficient received amounts, and balance deficits are
rejected. Source OFT nonces are retained as authenticated evidence in each
intent; because they are path-scoped, gaps and out-of-order execution are
valid.
Instance and persistent intent/replay storage receive explicit TTL extensions
on entrypoints, reads, and writes. If archival nevertheless occurs, the
committed sponsor restores the intent key in the transaction footprint and
calls `restore_intent` with the original sponsor reference before claiming or
cancelling.

The admin-authenticated pause stops new compose liabilities and claims while
leaving sponsor cancellation and expired recovery available. The immutable
inbox requires a replacement deployment and explicit versioned migration for
schema, trust boundary, route, or pause-policy changes; no balance sweep or
migration is authorized here. An independent reviewer must assess whether
this configured-OFT trust-boundary treatment closes the delivery-provenance
finding, given that no Soroban receiver can distinguish ambient fungible
transfers from official OFT transfers, before real funds are accepted.

The inbox deliberately does not call Campaign Escrow. A bridge message proves
the authenticated source path, but cannot provide the Soroban sponsor
authorization needed by `CampaignEscrow::fund` or establish the original
funder needed by its refund invariant. Automatic funding would weaken those
guarantees. Instead, the exact sponsor must authorize an inbox `claim` (or
`cancel`); after claiming, the sponsor performs a separate native
`CampaignEscrow::fund` call. Expired pending liabilities can be released
permissionlessly only to the exact committed sponsor; no owner recovery
address exists. Bridge messages cannot complete/refund campaigns or allocate
contributor pools.

The installed package metadata was checked before implementation: there is no
official Stellar OFT package in `node_modules`, only the OApp/endpoint/composer
surface. No OFT interface is fabricated. See the funding inbox README for the
payload and lifecycle details. The package has its own SDK 25.1.1/Rust 1.90
toolchain and is not part of the SDK 21/Rust 1.81 deployment boundary.

---

## Building

### Toolchain requirement — CRITICAL

Campaign Escrow, Contributor Pool, and Reputation Anchor **must compile with
Rust 1.81**. Rust 1.82+ produces WASM that soroban-sdk 21.x / stellar-cli 27
rejects at deploy-time:

```
HostError: Error(WasmVm, InvalidAction)
Translation error: "reference-types not enabled: zero byte expected"
```

This is a rustc/LLVM codegen issue with `wasm32-unknown-unknown` targets in newer compilers. `-C target-feature=...` flags do not help; the fix is an older compiler.

### Build commands

```bash
# Campaign Escrow
cd contracts/campaign_escrow
cargo +1.81 build --target wasm32-unknown-unknown --release

# Reputation Anchor
cd contracts/reputation_anchor
cargo +1.81 build --target wasm32-unknown-unknown --release

# Contributor Pool
cd contracts/contributor_pool
cargo +1.81 build --target wasm32-unknown-unknown --release

# LayerZero Receipt Adapter (isolated official SDK 25 toolchain)
cd contracts/layerzero_receipt_adapter
cargo +1.90 build --target wasm32v1-none --release
```

### Running tests

```bash
# Campaign Escrow (4 tests)
cd contracts/campaign_escrow
cargo test

# Reputation Anchor (6 tests)
cd contracts/reputation_anchor
cargo test

# Contributor Pool
cd contracts/contributor_pool
cargo test

# LayerZero Receipt Adapter
cd contracts/layerzero_receipt_adapter
cargo +1.90 test
```

### Deploying (stellar-cli 27)

```bash
# Upload and deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/<name>.wasm \
  --source <your-keypair-alias> \
  --network testnet
```

---

## Repository structure

```
contracts/
  campaign_escrow/
    Cargo.toml
    src/
      lib.rs    — contract implementation
      test.rs   — 4 integration tests
  reputation_anchor/
    Cargo.toml
    src/
      lib.rs    — contract implementation
      test.rs   — integration tests
  contributor_pool/
    Cargo.toml
    src/
      lib.rs     — immutable contributor manifest and self-claim implementation
      test.rs    — pool lifecycle and replay-protection tests
  layerzero_receipt_adapter/
    Cargo.toml
    src/
      lib.rs     — optional LayerZero V2 final-state receipt OApp
      test.rs    — authentication, replay, ordering, and payload tests
LICENSE
README.md
```

---

## License

MIT © 2024–2026 TogetherFi
