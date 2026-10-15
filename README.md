# TogetherFi × Stellar: Soroban Contracts

## Grant preparation status

Campaign Escrow v2 and both dedicated contributor pools are **deployed and
initialized on Stellar Testnet** for the architecture team's requested validation.
Both role accounts have verified 2-of-3 configurations and disabled setup masters.
Quorum-authorized completion is confirmed with two verified member signatures; no production readiness,
mainnet deployment or real-funds activity is claimed or authorized.
Historical v1 IDs remain separately labeled below.

**Pending signer update:** the owner selected Zyad to replace Ace on both testnet
role accounts, retaining 2-of-3. The existing roster remains active until the
existing quorum signs and the replacement is confirmed. See the
[testnet signer-rotation request](docs/testnet-signer-rotation.md).
Med Amine is a technical reviewer, not a signer.
See the [new native-only review handoff](docs/review-handoff.md) for the cleanup
boundary, requested reviews and remaining signer evidence.

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
TogetherFi is built by a human development team that was building the platform
before adopting AI tools. AI assists the team's engineering work; it is not the
sole builder or a substitute for human review.
The owner confirms a baseline of 3 hours/day including weekends, or 21 hours/week,
with extra weekend time potentially bringing availability to approximately 25 hours/week.
This is one shared allocation, not an allocation per work package.
The owner confirms separate human peer review by Zyad Rehioui (INKY), another
team developer. Med Amine has supplied a scoped review of the earlier PR.
Neither review is a formal audit, and the earlier review is not approval of this
updated snapshot. Fresh review by Med Amine and Zyad is requested.
King is a co-signer, not a code reviewer. Zyad is also the intended replacement
for Ace in the testnet signer group; that rotation is pending.
Production decisions are led by Ayoub; this does not authorize
mainnet deployment or real-funds operations.
[King's supplied professional profile](https://www.linkedin.com/in/dan-dean-ver-king-ramos-56073539a).

The future grant work plan totals 758 hours over 26 weeks, approximately
29.2 aggregate hours/week. Named allocations must reflect each contributor's
actual confirmed commitment; none are inferred from a review agreement.
Historical TogetherFi platform engineering hours are not Stellar Testnet hours.

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
tests the three retained native packages, and builds their WASM with Rust 1.81.0.
The workflow has three independent contract jobs and one offline Node-guard job.
It retains verification artifacts. A reference copy is supplied as
`docs/proposed-ci-workflow.yml`.
See [CI runs](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml)
for the result of the exact new PR commit. Earlier five-contract/six-job runs
belong to the historical broader source tree and do not prove this cleanup passed.
A configured workflow is not passing evidence, and passing CI alone does not
prove its WASM bytes equal the previously deployed locally built binaries.

The selected settlement-control mechanism is a native Stellar account multisig
with a **2-of-3** policy. The current on-chain roster is
**Ayoub Arbani, Zyad, and King** on both dedicated role accounts.
The replacement was confirmed on 15 October 2026 in ledger 5207471:
[rotation confirmation](evidence/testnet-signer-rotation/confirmation.json).
The historical deployment manifest retains the original Ayoub, Ace and King roster.
Three independently held Stellar signing keys are required, not EVM addresses
or three keys held by the backend.
Both account configurations are **verified on testnet**: signer weights 1/1/1,
low/medium/high thresholds 2/2/2, setup master weight 0. Disabled setup-key attempts
were rejected by the network. The historical member 1-of-3 rejection and 2-of-3
completion remain valid evidence for that earlier roster, not Zyad's participation.
A separate one-test-XLM campaign is active for a
[fresh Zyad and King release test](docs/testnet-zyad-king.md).
Their wallet signatures, fresh below-quorum rejection and successful release
are still pending. Mainnet needs different owner-provided wallets.
Governance custody must also be independently threshold-controlled so the backend
cannot bypass settlement quorum by rotating the settlement role to its own key.
See [testnet acceptance checklist](docs/testnet-acceptance.md).

The public review snapshot is Stellar-native: sponsors fund campaigns directly
on Stellar. Optional bridge components and their dependency trees are absent.
No cross-chain funding, USDC liquidity or replenishment mechanism is claimed.

This repository contains the native Soroban contracts for the
[TogetherFi](https://togetherfi.io) Stellar integration.

---

## Existing v1 deployed contracts (Stellar testnet, historical)

The addresses and transactions below are **v1 historical evidence only**.
They are not the v2 implementation and do not describe a new deployment.

| Contract | Address |
|---|---|
| Campaign Escrow | `CDEAOPTIFKCOFOOHNXXFLF3WAUSTXZKMIXN7NM5OM2BO3CLBX2AND34N` |
| Reputation Anchor | `CBJVDSS6VUWNORYG2W7ZMHPGMCBXQN5A2CKK5OSS7L2WFR6RSFLGQILC` |

---

## Campaign Escrow v2 (`contracts/campaign_escrow`): testnet validation

Testnet contract: `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`.
Test fixture asset is free seven-decimal XLM, not production USDC.

Holds the configured settlement asset for a TogetherFi creator campaign and releases it atomically on
completion with the platform's fixed payout split:

| Recipient | Share |
|---|---|
| Creator wallet | 82% |
| Platform treasury | 10% |
| NFT contributor pool | 5% |
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

## Contributor Pool (`contracts/contributor_pool`): testnet validation

Both separate instances are deployed and initialized:
- 5% NFT contributor pool: `CAUPWNNQLQLURIJJTFW7LURJRD2OLGV4R5E6LNSUJXKSESRVGO54YP76`.
- 2% community: `CD6TNSBSLAPXE4JXDBNTNEMMXH7DAWGUA3TR5HMP4QQIHCYM7544ED25`.
Their first authenticated campaign credits are confirmed: 0.05 and 0.02 test XLM.

Deploy **two separately initialized instances**: one for the 5% NFT
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
two authorities publish: NFT contributors need verified ownership and
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

Split verified on-chain: 82.5% creator / 10% platform treasury / 5% revenue pool / 2.5% referrer. Escrow drained to zero.

> Note: XLM native asset used as USDC stand-in for the testnet test.
> This historical stand-in does not select the v2 asset/issuer. The final asset
> requires separate confirmation; no mainnet deployment is authorized here.

---

## Reputation Anchor (`contracts/reputation_anchor`)

Stores creator TogetherScores permanently on Stellar as the authoritative
on-chain identity record. The historical contract schema names one optional
external reference field `arbitrum_tx_hash`. Its name is retained only for
compatibility with the deployed historical interface. It is optional application
metadata, not a funding requirement, bridge component or authenticated proof.

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

---

## Building

### Toolchain requirement: CRITICAL

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

```

### Running tests

```bash
# Campaign Escrow
cd contracts/campaign_escrow
cargo +1.81.0 test --locked

# Reputation Anchor
cd contracts/reputation_anchor
cargo +1.81.0 test --locked

# Contributor Pool
cd contracts/contributor_pool
cargo +1.81.0 test --locked
```

### Reproducible evidence bundle

Install the exact Rust `1.81.0` toolchain and the
`wasm32-unknown-unknown` target, then run:

From the Soroban repository root (or the application subtree), run:

```bash
for package in campaign_escrow contributor_pool reputation_anchor; do
  bash scripts/verify-contract.sh "$package" 1.81.0 wasm32-unknown-unknown
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
      lib.rs    : contract implementation
      test.rs   : escrow lifecycle, authorization, and immutable-policy tests
  reputation_anchor/
    Cargo.toml
    src/
      lib.rs    : contract implementation
      test.rs   : integration tests
  contributor_pool/
    Cargo.toml
    src/
      lib.rs     : immutable contributor manifest and self-claim implementation
      test.rs    : pool lifecycle and replay-protection tests
LICENSE
README.md
```

---

## License

MIT © 2024-2026 TogetherFi
