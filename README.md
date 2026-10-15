# TogetherFi × Stellar — Soroban Contracts

## Grant preparation status

Campaign Escrow v2 and both dedicated contributor pools are **deployed and
initialized on Stellar Testnet** for the architecture team's requested validation.
Both role accounts have verified 2-of-3 configurations and disabled setup masters.
Quorum-authorized completion is confirmed with two verified member signatures; no production readiness,
mainnet deployment or real-funds activity is claimed or authorized.
Historical v1 IDs remain separately labeled below.

Current [testnet evidence](docs/testnet-deployment-evidence.md) includes the v2 ID,
transactions, deployed WASM verification and pending acceptance gates.
See [wallet signing instructions](docs/testnet-signing.md) for the quorum test.

Confirmed lifecycle proofs:
[v2 deployment](https://stellar.expert/explorer/testnet/tx/86f5b65bfb589121aa44e207922c18ce3fcc7a0422cd7dbb68dc8c8d593b3114),
[2-of-3 handoff](https://stellar.expert/explorer/testnet/tx/9bb97971a294acc396362834eefdcc1cb96c2fe8916be8e8294a4bc482451d29),
[sponsor funding](https://stellar.expert/explorer/testnet/tx/53c2e5ed884c8575507760640ed1d05749f22388418468ee28cafa6d674bc0e3),
[sponsor activation/approval](https://stellar.expert/explorer/testnet/tx/8ecd390622067c25610596b0ce78101c5db6d956293119109b176225dc76b729).
[Quorum-authorized release](https://stellar.expert/explorer/testnet/tx/ec8ca3af04c870739b201d6746faf4fb7ded540f1cf5e8336215908803518404)
is confirmed with exact 82/10/5/2/1 payout deltas and reconciled liability.
The real 30-day timeout/refund proof remains pending.

This working tree is pinned to upstream baseline commit
`6fd30883bf8c305eb8dd8a8a8fabe7226171332b` in `UPSTREAM_COMMIT`.

[![Tests](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml/badge.svg)](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml)

## Engineering ownership and AI assistance

Ayoub Arbani is the CEO/Developer and Soroban technical owner:
[personal GitHub](https://github.com/AGDAO/) and
[LinkedIn](https://linkedin.com/in/ayoub-arbani/).
AI coding assistants produced most implementation under his direction.
AI-assisted implementation and local review are not an independent human audit.
The draft grant plan proposes approximately 17 hours/week across remaining core
Soroban work, not 17 hours per work package. Actual weekly availability and parallel
commitments still require owner confirmation. Ayoub Arbani is the sole current
reviewer as well as the implementation owner. Ace and King are security co-signers,
not reviewers. Owner review is not independent review; no independent human
escrow v2 review or external audit is claimed.
[King's supplied professional profile](https://www.linkedin.com/in/dan-dean-ver-king-ramos-56073539a).

## Architecture-team acceptance evidence

The fixed policy is **82/10/5/2/1**. `complete(campaign_id, operation_version)`
has no split or destination argument, and there is no split setter.
Only the stored sponsor can commit creator/referrer at activation; they cannot
be replaced afterward. Governance cannot authorize settlement by itself.

Direct tests in [campaign escrow tests](contracts/campaign_escrow/src/test.rs):
- `admin_alternative_split_calls_fail_without_changing_campaign_or_balances`
- `admin_cannot_select_or_replace_sponsor_committed_beneficiaries`
- `governance_is_not_the_settlement_authority`
- `exact_split_sends_each_leg_to_its_approved_destination`

The verification script uses Rust-compatible committed lockfiles (`--locked`),
tests all five packages, and builds their WASM with pinned toolchains.
The active expanded workflow tests and builds all five contracts independently,
retains verification artifacts, and runs the offline Node approval guards.
It is installed on the review branch using explicitly authorized workflow-write
access. A reference copy is supplied as `docs/proposed-ci-workflow.yml`.
See [current CI runs](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml?query=branch%3Aescrow-v2-ci-security-evidence-20261004)
for the actual result; workflow configuration alone is not passing evidence.
The previous CI and published lockfiles resolved newer edition-2024 dependencies
incompatible with Rust 1.81, failing before contract tests ran. A configured workflow is not
a passing run; use the linked Actions result as the current CI evidence.

The selected settlement-control mechanism is a native Stellar account multisig
with a **2-of-3** policy, matching the owner's Arbitrum model. The selected
signer holders are **Ayoub Arbani, Ace, and King**.
Three independently held Stellar signing keys are required, not EVM addresses
or three keys held by the backend.
Both account configurations are **verified on testnet**: signer weights 1/1/1,
low/medium/high thresholds 2/2/2, setup master weight 0. Disabled setup-key attempts
were rejected by the network. Actual member 1-of-3 rejection occurred at RPC
ingestion, and 2-of-3 completion is ledger-confirmed. Mainnet needs different
owner-provided wallets.
Governance custody must also be independently threshold-controlled so the backend
cannot bypass settlement quorum by rotating the settlement role to its own key.
See [testnet acceptance checklist](docs/testnet-acceptance.md).

Arbitrum-triggered Stellar payments are **excluded from the current grant scope**.
No Stellar-side USDC liquidity or replenishment mechanism is claimed.

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

## Campaign Escrow v2 (`contracts/campaign_escrow`) — testnet validation

Testnet contract: `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`.
Test fixture asset is free seven-decimal XLM, not production USDC.

Holds the configured settlement asset for a TogetherFi creator campaign and releases it atomically on
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
| `fund(campaign_id, sponsor, amount)` | sponsor signs | Transfer configured asset sponsor → contract |
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

## Contributor Pool (`contracts/contributor_pool`) — testnet validation

Both separate instances are deployed and initialized:
- 5% MOFO: `CAUPWNNQLQLURIJJTFW7LURJRD2OLGV4R5E6LNSUJXKSESRVGO54YP76`.
- 2% community: `CD6TNSBSLAPXE4JXDBNTNEMMXH7DAWGUA3TR5HMP4QQIHCYM7544ED25`.
Their first authenticated campaign credits are confirmed: 0.05 and 0.02 test XLM.

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
> This historical stand-in does not select the v2 asset/issuer. The final asset
> requires separate confirmation; no mainnet deployment is authorized here.

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
    pub score:            u32,    // 0-1000 (TogetherScore v2)
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

### Reproducible evidence bundle

Install the exact Rust `1.81.0` and `1.90.0` toolchains and the
`wasm32-unknown-unknown` / `wasm32v1-none` targets, then run:

From the Soroban repository root (or the application subtree), run:

```bash
for package in campaign_escrow contributor_pool reputation_anchor; do
  bash scripts/verify-contract.sh "$package" 1.81.0 wasm32-unknown-unknown
done
for package in layerzero_receipt_adapter layerzero_funding_inbox; do
  bash scripts/verify-contract.sh "$package" 1.90.0 wasm32v1-none
done
```

The script uses committed lockfiles with `--locked`, tests each package, builds
its WASM with the required target, and writes transcripts and hashes to
`evidence/current/<package>/`. These are local
reproducibility artifacts only; they are not deployment, testnet, or live-route
proof.

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
      test.rs   — escrow lifecycle, authorization, and immutable-policy tests
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

MIT © 2024-2026 TogetherFi
