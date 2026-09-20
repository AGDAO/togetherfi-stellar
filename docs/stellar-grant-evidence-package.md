# TogetherFi Stellar Grant Evidence Package

**Audience:** founder and architecture team  
**Evidence cut-off:** repository state and tracked documentation reviewed 13 September 2026  
**Use:** copy-ready technical evidence dossier; not an audit, deployment approval, or production-readiness statement

## Executive accuracy statement

The repository contains a substantial Soroban implementation, EVM sponsor-funding and
receipt components, and application reconciliation code. The current v2 escrow,
Funding Inbox, receipt adapter, sponsor-funding source contract, and backend route are
**implemented and locally tested, not deployed/live.** Local tests and review records
are not real cross-chain proof. There is no claimed real cross-chain transaction,
GUID, live route, or sponsor-funded v2 campaign.

The historical Stellar testnet contracts are v1 evidence only. They are not the
current v2 escrow and do not establish the current payout policy. The reviewed
Arbitrum-Sepolia-to-Stellar-testnet LayerZero route is **NO-GO**: the published
LayerZero registry has no Stellar testnet Endpoint/EID deployment and no reviewed
OFT deployment for that path. No sponsor funds should be accepted until the
route, asset, identities, manifest, and operational controls are independently
verified.

This report intentionally excludes private keys, credentials, secrets, internal
remotes, private infrastructure addresses, customer information, and unnecessary
implementation detail.

## 1. Repository and provenance

### 1.1 Repository facts

| Item | Verified repository evidence |
|---|---|
| Public source repository | `https://github.com/AGDAO/togetherfi-stellar` |
| Independently checked visibility | Anonymous HTTPS returned HTTP 200 for the repository and immutable commit on 13 September 2026; the unauthenticated GitHub API returned the same full commit hash and message. |
| Immutable current-source commit | `https://github.com/AGDAO/togetherfi-stellar/commit/0493b2cee72f52d38872152d26bf320d1281ef78` |
| Working-tree source | `contracts/togetherfi-stellar/` in this application repository, with `UPSTREAM_COMMIT` set to `6fd30883bf8c305eb8dd8a8a8fabe7226171332b`. |
| Application repository URL | None is locally documented in the reviewed files. Do not infer or publish a URL from local remotes. |
| Release provenance | `RELEASE.md` records the exact upstream baseline, MIT license, founder responsibility and maintenance statement, commit attribution boundary, AI-assistance disclosure, release scope, and exclusions. |
| Public-source status | The current v2 Escrow, Contributor Pool, Funding Inbox, receipt adapter, selected EVM LayerZero boundary source, tests, lockfiles, and vendored dependencies are published at the immutable commit above. `UPSTREAM_COMMIT` remains the earlier baseline, not the current release commit. |

The local Git remote list contains private/internal Replit remotes. They are
deliberately not reproduced here. Git author names below are the names recorded in
the local history; they are not a substitute for a founder responsibility statement
or independent authorship verification.

### 1.2 Relevant history, grouped by implementation area

Only commits materially relevant to this dossier are listed. Hashes, authors, and
dates are copied from local Git history; the Area column is a scoped summary of
the relevant changes rather than the exact Git subject.

**Soroban baseline and escrow/security**

| Commit | Author | Date (UTC) | Area |
|---|---|---|---|
| `842dfa6996ace851a57695c8ad4030b50a47c2b7` | defiholic | 2026-09-10 | Added the public Soroban baseline, Campaign Escrow, Reputation Anchor, tests, workflow, license, and reproducible-build boundary. |
| `001d06048b1fd4ae19030a738ec4d6eff85876b7` | Replit Agent | 2026-09-11 | Implemented escrow security changes, Contributor Pool contracts, contributor routes, and Stellar settlement integration changes. |
| `86b7e6f91f9ba632d8342f0747b22b40fa8f483a` | Replit Agent | 2026-09-12 | Updated the escrow redesign and implementation-boundary documentation. |

**LayerZero, EVM source route, and Funding Inbox**

| Commit | Author | Date (UTC) | Area |
|---|---|---|---|
| `78742b07d71924f99946432ff7381bd0a17d70cc` | Replit Agent | 2026-09-12 | Added cross-chain funding service lifecycle, route APIs, receipt outbox/runtime adapters, and backend tests. |
| `993cce3fc6927554a197e17fb4d65ef8ca1659c0` | Replit Agent | 2026-09-12 | Added/updated the LayerZero Funding Inbox, EVM sponsor-funding contract, reviewed OFT wrapper, security review record, and tests. |
| `47c469322aee54179ea2d65a7a4404677399433d` | defiholic | 2026-09-12 | Blocked the unproven LayerZero route at runtime and recorded the testnet NO-GO. |

**Funding model and historical EVM foundation**

| Commit | Author | Date (UTC) | Area |
|---|---|---|---|
| `fabd8889eee9793afbacdf11cf3f6a8af8227c6a` | defiholic | 2026-08-11 | Git subject records Arbitrum Escrow/Distributor v2 with the 82/10/5/2/1 model and 36 passing tests; this is existing EVM foundation, not new Stellar grant work. |
| `78a038ef61b2fdbaa6c672fc1446562fad261ead` | Replit Agent | 2026-07-16 | Updated Stellar campaign handling for native Soroban escrow settlement. |
| `ae18326a5f368883c8d65b6225d94200f54e3fac` | Replit Agent | 2026-07-16 | Added Stellar score-verification and transparency integration. |

The last table's EVM commit is retained only as existing capability evidence; it
is not required to prove the current Stellar implementation or a new grant
deliverable.

## 2. Soroban engineering and complete inventory

### 2.1 Soroban contracts and components

| Component and source location | Purpose and current state | Tests/evidence | Deployment and safe disclosure |
|---|---|---|---|
| `contracts/togetherfi-stellar/contracts/campaign_escrow/` | Current CreatorFi-only v2 liability-bearing escrow. `fund`, sponsor-only `activate`, settlement-authority `complete`, public `refund_expired`, two-step role rotation, pause, TTL extension, operation consumption, and exact split. Implemented locally; no v2 deployment. | Current `src/test.rs` declares 12 tests covering authorization, split, expiry, pause, rotation, deficits, isolation, TTL, and accounting. Tracked README's older “4 tests” statement is stale relative to the current file; no standalone execution transcript is tracked in this dossier. | Show source, pinned commit, tests, and a redacted build log. Do not present v2 as the historical deployed contract. |
| `contracts/togetherfi-stellar/contracts/contributor_pool/` | Separate immutable-manifest pools: one 5% MOFO allocation and one 2% community/revenue allocation. Dual authorities publish exact recipients/amounts; recipients self-claim; expiry carries unclaimed reservations forward. Implemented locally; no deployment IDs. | Current `src/test.rs` declares 8 tests, including manifest immutability, dual authority, funding, expiry, pause, role, claim, and direct-transfer isolation. Execution result is not separately recorded. | Source and invariants can be shared; keep eligibility evidence, signer identities, and operational policy private unless requested. |
| `contracts/togetherfi-stellar/contracts/reputation_anchor/` | Admin-authorized score records, public reads/batch reads, `ScoreAnchored` events. Separate from Classic Stellar `manageData` account anchoring. Implemented; current source has 11 declared tests; no new deployment claimed. | README's six-test description is historical/documentary and does not supersede the current test file. | Show source/API and a future verified transaction; never call Classic account data Soroban. |
| `contracts/togetherfi-stellar/contracts/layerzero_receipt_adapter/` | SDK-25/Rust-1.90 isolated OApp for versioned, domain-separated, financially inert final-state receipts. It cannot settle, refund, or move campaign funds. Implemented locally; not live. | Current `src/tests.rs` declares 5 tests for payloads, malformed/status rejection, peer/nonce checks, mock endpoint delivery, and quote/send receipt behavior. | Show interface, isolation boundary, tests, and later manifest; no endpoint/peer/transaction is currently claimable. |
| `contracts/togetherfi-stellar/contracts/layerzero_funding_inbox/` | SDK-25/Rust-1.90 Soroban Inbox. Authenticates configured OFT compose provenance and records a pending liability; `claim` is sponsor-authorized and never calls escrow. Includes cancel, expiry recovery, pause, TTL restoration, and duplicate guards. Implemented locally; not live. | Independent review record: **10/10**, with wrong route/amount/queue, replay, ambient-balance, pause, TTL, and recovery coverage. | Show source, ABI/payload schema, and review result with the explicit configured-OFT residual; do not present a bridge delivery as funding. |
| Vendored OApp/endpoint tree | Pinned dependency surface for the two LayerZero Soroban packages, not a TogetherFi deployment. | Dependency/toolchain boundary reviewed. | Share dependency versions/lockfiles as needed; do not imply LayerZero registry deployment. |

The Campaign Escrow, Contributor Pool, and Reputation Anchor use Soroban SDK
21.7.6/Rust 1.81. The LayerZero packages use Soroban SDK 25.1.1/Rust 1.90 in
isolated workspaces. The README states the receipt adapter target is
`wasm32v1-none`; the older escrow family uses `wasm32-unknown-unknown`. A release
must record the exact target, lockfile, compiler, source commit, and WASM hash for
each package. No WASM hash or new deployment manifest is currently evidence here.
The tracked source is suitable for future independent source verification, but
verification is not complete for the historical Reputation Anchor and has not
been submitted for the new v2 or LayerZero artifacts.

### 2.2 EVM components

| Component and path | Role and state |
|---|---|
| `mofo-deploy/contracts/ArbStellarSponsorFunding.sol` | Arbitrum source-side funding intent contract. It authenticates sponsor/terms, allowlists token and adapter route, builds the TFZF compose instruction, and emits funding/submission/refund evidence. It has no Stellar payout/refund authority. Implemented and locally tested; undeployed. |
| `mofo-deploy/contracts/ReviewedOFTAdapter.sol` | Non-upgradeable local wrapper at the reviewed official IOFT ABI boundary. It pins underlying identity/route assumptions and supports the two locally tested approval modes (`OFT=false`, `OFTAdapter=true`). It is not an official live deployment. |
| `mofo-deploy/contracts/ArbStellarReceiptReceiver.sol` | Arbitrum peer receiver for receipt metadata only. It authenticates endpoint/peer/EID and rejects replay/out-of-order payloads; it has no value-moving method. Implemented/tested locally; undeployed. |
| `mofo-deploy/contracts/vendor/layerzero-v2/` | Vendored interface/provenance boundary (`IOFT`, Endpoint V2, receiver/composer interfaces). Not a live endpoint or peer configuration. |
| Existing EVM CreatorFi/SocialFi foundation | The tracked plan names RewardEscrowV3, RewardDistributorV2, AttestationVerifierV2, TogetherFiDistribution, wallet/oracle/revenue-pool components and associated tests. These are capability evidence and existing baseline, not new Stellar grant deliverables. Exact deployment claims require their manifests and receipts, which are not asserted here. |

### 2.3 Backend/application inventory

Relevant implementation is in `server/` and `tests/`:

- `crossChainFundingIntent.ts` is a non-custodial audit/reconciliation ledger. It
  accepts observed hashes and states; it does not broadcast, hold keys, claim an
  Inbox liability, or call escrow.
- `routes-cross-chain-funding.ts` exposes authenticated sponsor preparation,
  quote/submission recording, destination observation, and claim-XDR preparation.
  The explicit reviewed-deployment gate and complete identity checks remain
  required.
- `stellarFundingInbox.ts`, `arbStellarSponsorFunding.ts`, and
  `arbStellarSponsorFundingAbi.ts` are read/build boundaries, not a signer wallet.
- `layerZeroReceiptOutbox.ts` and `layerZeroRuntimeAdapters.ts` provide a durable,
  best-effort receipt outbox, accepted-transaction reconciliation, finality and
  canonicality checks, and no fallback into settlement.
- `stellarSorobanEscrow.ts`, `stellarSettlement.ts`, `chainRouter.ts`, and
  contributor-pool services integrate native Stellar funding/settlement and
  preserve chain isolation.
- `server/migrations/boot.ts` installs/updates the relevant durable ledger
  structures. Database rows and synthetic/dry-run identifiers are not on-chain
  proof.

## 3. Current versus historical escrow security

### 3.1 Historical v1 testnet baseline

The tracked README records these full historical Stellar testnet IDs:

| Contract | Historical ID | Status |
|---|---|---|
| Campaign Escrow v1 | `CDEAOPTIFKCOFOOHNXXFLF3WAUSTXZKMIXN7NM5OM2BO3CLBX2AND34N` | Historical testnet evidence only; not v2 and not current policy evidence. |
| Reputation Anchor | `CBJVDSS6VUWNORYG2W7ZMHPGMCBXQN5A2CKK5OSS7L2WFR6RSFLGQILC` | Historical testnet evidence; source-verification submission is recorded as pending. |

The v1 README records deployment transaction
`54b6a37c3b95c86135aa8ae369f761f99a4204125bc025363bf2d3171101f553`,
initialization transaction
`ff9e9c6347fd4acb86986dfd52e87a649b87b350338a096bcbb6e30c5b05b1bd`, fund
transaction
`ebced8761f76e5e1bbdb5d0fe0859f095ce999e2fb5487358474399484d25c89`, and
completion transaction
`268ad813ee9192a9a6838b03173e3403d83df7bf1ab3cf17ae2c8790f574496b`.
The same document reports the v1 on-chain split as 82.5% creator, 10% platform
treasury, 5% revenue pool, and 2.5% referrer, with escrow drained to zero, using
XLM as a USDC stand-in. These facts must not be used to describe v2.

### 3.2 Current v2 implementation

The current source is a fresh immutable-design candidate, not a migrated or
upgraded version of v1. It stores the original sponsor, amount, token, timestamps,
terms, recipient, referrer, status, operation version, total liability, and
consumed operations. It has:

- Sponsor authorization for `fund` and `activate`; the contract transfers exact
  base units from sponsor to escrow.
- Separate governance and settlement addresses. Governance initializes, pauses,
  unpauses, and proposes role changes. The proposed address must accept a role
  change in a second authorized step.
- Settlement authority authorization for `complete`; it cannot choose a new
  recipient or redirect a refund.
- `refund_expired` callable by anyone at/after expiry, while paused, with the
  destination always read from stored sponsor state.
- Atomic state/liability update and payout/refund operation with terminal-state
  and consumed-operation guards.
- Fixed pool destinations configured at initialization and authenticated
  contributor-pool credit calls.
- No timelock implementation. Two-step role transfer is present; a production
  multisig/timelock policy is an open deployment/governance decision.
- No owner rescue of the configured campaign asset while liabilities exist; the
  documented recovery concept is limited to separately reviewed unsupported
  accidental tokens.

The contract separates settlement from governance in code, but it does not itself
enforce a multisig, threshold, hardware signer, or timelock. A deployment that
uses one governance key or one settlement key therefore retains a single-key
compromise risk. Governance can pause/rotate but cannot select an ordinary payout
destination; settlement can complete only committed terms. These operational
controls and their rotation/recovery drill remain release gates.

The implementation is **implemented** and has local source-level test coverage.
The repository review records local PASS for the LayerZero/EVM/Soroban Inbox and
backend tracks, not a live escrow deployment. It is not **testnet-ready for
accepting funds**: fresh deployment, exact asset/issuer, source verification,
independent escrow review, manifest, role provisioning, monitoring, and
end-to-end acceptance remain gates. It is **not yet deployed**. Existing v1
testnet IDs do not change these labels.

## 4. LayerZero implementation and security boundary

LayerZero is deliberately split into two non-authoritative functions:

1. The Stellar receipt adapter sends a versioned, domain-separated final-state
   receipt only after native Stellar completion/refund. The Arbitrum receiver
   records metadata and cannot call payout, refund, distribution, token, or
   arbitrary external methods.
2. The optional OFT sponsor route delivers an allowlisted asset to the separate
   Funding Inbox. It creates a pending liability only. The exact sponsor must
   claim it and then authorize native escrow funding.

The source pins the official Stellar OApp dependency tree at version 1.2.55, the
LayerZero packages at Soroban SDK 25.1.1/Rust 1.90, and the receipt/funding
workspaces separately from the SDK 21/Rust 1.81 escrow family. The EVM package
pins `@layerzerolabs/oft-evm` 4.0.1 for ABI compatibility tests; its published
OFT artifacts are abstract, so the local `ReviewedOFTAdapter` is a reviewed
boundary, not a deployed official OFT. Endpoint, peer, EID, Executor, DVN,
options, asset, and signer values are configuration inputs, not current
deployment evidence. Local endpoint and adapter tests use mocks or injected
observations; no supported Stellar testnet LayerZero environment is configured.
The route review records the published Arbitrum Sepolia Endpoint V2 as
`0x6EDCE65403992e310A62460808c4b910D972f10f4` (chain ID `421614`, EID
`40231`) and the published Stellar mainnet Endpoint V2 as
`CCQLLRE5JBAWYCW3KTWOIWLMFDUOKROQVZNSALQMGOSXNW3ERUOWTZGK` (EID `30600`).
Those published identities are reference data only: no Stellar testnet Endpoint
or EID, OFT deployment, peer, Executor, DVN, or enforced-options configuration
is claimed for TogetherFi.

The security review is conditional/local PASS (Solidity 18/18, Inbox 10/10,
backend 35/35, runtime 11/11, route/runtime 16/16). It explicitly accepts the
configured-OFT amount trust-root residual described in Section 11 and keeps
runtime disabled until exact live configuration, testnet proof, a signed
manifest, and an operational pause/recovery drill exist.

## 5. Arbitrum-to-Stellar funding and custody lifecycle

### 5.1 Native Stellar campaign

1. A sponsor chooses Stellar and signs `CampaignEscrow::fund` from the sponsor
   wallet. The contract receives and records campaign-specific principal.
2. The sponsor signs `activate`, committing creator, required referrer, terms, and
   operation version. The campaign cannot be treated as funded merely because an
   application row exists.
3. The settlement authority authorizes/submits `complete` before the 30-day
   expiry. The contract distributes the committed legs atomically.
4. At/after expiry, the sponsor or a public caller can submit `refund_expired`; the
   contract returns the remaining balance to the stored original sponsor.
5. The application reconciles prepared/submitted/confirmed/unknown/rejected states;
   chain state is authoritative after confirmation.

At all times before terminal settlement, the Soroban escrow holds a liability, not
an application wallet balance. The application database holds references and
accounting records only. Application servers, relayers, governance, and settlement
authority must not hold or borrow campaign principal or sponsor signing keys.

### 5.2 Optional Arbitrum-to-Stellar sponsor route

1. The sponsor authorizes a funding intent on Arbitrum. The source contract and
   allowlisted OFT transport the specified asset through the configured LayerZero
   path.
2. LayerZero delivery to `FundingInbox` records a **pending inbox liability**.
   Tokens in the Inbox are not Campaign Escrow principal and cannot activate a
   campaign.
3. The exact committed sponsor verifies identity, amount, asset/domain, deadline,
   and campaign/version, then signs a separate Inbox claim (or cancel/recovery).
4. After a confirmed claim, that same sponsor signs a separate native Stellar
   `CampaignEscrow::fund` transaction.
5. Only confirmed native escrow funding can activate or settle the campaign.

The Inbox never calls escrow. The LayerZero receipt adapter is a separate,
post-finality metadata channel. A receipt, OFT delivery, source confirmation,
database state, or LayerZero status cannot complete, refund, fund, or pay a
campaign. LayerZero therefore cannot control campaign settlement funds.

### 5.3 Current route truth

The service and adapters are non-custodial and locally tested. The route is
disabled and **NO-GO for testnet** because the required official Stellar testnet
Endpoint/EID and reviewed OFT deployment are absent from the published registry.
No real cross-chain proof exists. The mainnet-only published Stellar endpoint
cannot be substituted for testnet evidence.

## 6. Stellar economics, accounting, and liquidity

### 6.1 Current v2 CreatorFi policy

The local v2 source uses a 10,000-unit denominator and this exact gross campaign
split:

| Leg | Share | Accounting treatment |
|---|---:|---|
| Creator | 82% | Participant/creator payout leg. |
| TogetherFi service treasury | 10% | Platform service-fee revenue, separate from campaign principal accounting. |
| MOFO contributor pool | 5% | Fixed contributor-pool liability, not automatic holder payment and not treasury revenue. |
| Community/revenue contributor pool | 2% | Separate fixed contributor-pool liability, not treasury revenue. |
| Verified referrer | 1% | Explicit address required at activation; no fallback recipient. |

The source computes each non-creator leg with integer floor division and assigns
the explicit remainder to the creator:

`creator = gross - service - MOFO - community - referrer`.

Therefore every completion leg sums exactly to gross amount and no rounding dust
is trapped. The minimum campaign amount is 100 base units so each 1%/2%/5% leg
is nonzero. The configured asset uses integer base units; the recorded owner
decision specifies a 7-decimal asset, but the official asset/issuer and network
contract are not yet confirmed. Do not call XLM or USDC the final live asset.
XLM was the historical testnet stand-in; any future asset claim needs a signed
network-specific manifest.

Pool allocations become available only through separately approved immutable
manifests. MOFO eligibility requires off-chain verification of ownership and
campaign-promotion evidence; community eligibility requires off-chain
verification of qualifying content and required UTC check-ins. The Soroban pool
does not verify those external facts. It enforces that governance and an
independent eligibility authority approve the same immutable recipient/amount
manifest, after which recipients self-claim. Claim windows are bounded at 30
days; unclaimed expired amounts carry forward to later contributor cycles and do
not move to governance. The on-chain manifest, authorization, claim, expiry, and
carry-forward controls are implemented locally; the eligibility evidence process
still requires documented operations and independent review.

There is currently no TogetherFi token. No token issuance, token liquidity, DEX
pool, shared settlement pool, or separate TogetherFi liquidity pool is present or
required by this architecture.

### 6.2 Liquidity answer

The source of Stellar campaign capital is the sponsor's campaign funding, either
directly from a Stellar wallet or, after the separate Inbox claim, from the
sponsor's native Stellar escrow funding transaction. This is campaign funding and
solvency, not platform liquidity. There is no founder-supported rebalancing pool,
initial liquidity size, or threshold to report. The team still must decide and
document the official asset/issuer, decimals, sponsor wallet flow, governance
custody, and operational treatment of an Inbox liability before enabling any route.

## 7. Test evidence and limitations

Counts below distinguish explicitly recorded review results from test functions
present in source. “NR” means no pass/fail/ignored execution count is recorded in
the tracked evidence; it must not be rewritten as zero. The same test may exercise
several properties, so rows must not be added together as a unique-test total.

| Suite | Passed | Failed | Ignored | What it proves | Path / reproducibility |
|---|---:|---:|---:|---|---|
| Solidity LayerZero/source route review | 18 | 0 | 0 | Reviewed wrapper boundary, exact pinned IOFT selectors, official approval modes, route identity and source funding behavior. | `cd mofo-deploy && npm run test:layerzero-clean` was reproduced on 13 September 2026: 69 Solidity files compiled and 18 tests passed. This is not deployment proof. |
| Soroban Campaign Escrow | NR (12 declared) | NR | NR | Current source-level authorization, split, expiry, pause, role, deficit, isolation, TTL, and accounting cases. | `contracts/.../campaign_escrow/src/test.rs`; run with the documented Rust 1.81 toolchain. No standalone pass transcript is tracked. |
| Soroban Reputation Anchor | NR (11 declared) | NR | NR | Current source-level score lifecycle, access control, batch reads, bounds, and profile isolation. | `contracts/.../reputation_anchor/src/test.rs`; README’s six-test note is older and not a current execution count. |
| Soroban Contributor Pool | NR (8 declared) | NR | NR | Manifest immutability, dual authority, claims, carry-forward, pause, role and direct-transfer isolation. | `contracts/.../contributor_pool/src/test.rs`; execution count not separately recorded. |
| Soroban Funding Inbox review | 10 | 0 | 0 | Inbox authentication, amount/route/queue checks, pending liability lifecycle, replay/duplicate rejection, TTL and pause/recovery. | `contracts/.../layerzero_funding_inbox/src/tests.rs`; final independent review record says 10/10, local/repository evidence only. |
| Soroban receipt adapter | NR (5 declared) | NR | NR | Versioned payload, malformed/status rejection, peer/nonce checks, and mock endpoint behavior. | `contracts/.../layerzero_receipt_adapter/src/tests.rs`; mock endpoint is not a live LayerZero endpoint. |
| Focused backend review record | 35 | 0 | 0 | Funding-intent lifecycle, finality/canonicality, source reorg/missing-receipt handling, durable halt, and route reconciliation. | The independent review record identifies 35/35 across its focused evidence set. This count overlaps the narrower rows below and is not a current file-count total. |
| Runtime adapter review record | 11 | 0 | 0 | Event decoding, accepted hashes, finalized source evidence, ABI fixture and receipt/Inbox identity boundaries. | The independent review record identifies 11/11 for its accepted fixture set. |
| Backend route/runtime review record after wrapper | 16 | 0 | 0 | API route authorization, exact calls, claim identity, route gates and wrapper-integrated runtime behavior. | The independent review record identifies 16/16 for its wrapper-integrated evidence set. |
| Current combined backend reproduction | 45 | 0 | 0 | Current funding-intent, receipt-outbox, runtime-adapter, and funding-route behavior, including reorg/finality, receipt disappearance, no-rebroadcast reconciliation, route halts, claim causality, and ABI fixtures. | `npx tsx --test tests/crossChainFundingIntent.test.ts tests/layerZeroRuntimeAdapters.test.ts tests/arbStellarFundingRoutes.test.ts tests/layerZeroReceiptOutbox.test.ts` was reproduced on 13 September 2026. |
| Integration/security/replay/wrong-peer/wrong-inbox/under-delivery/deadline/reorg/pause | Included above | Included above | Included above | These are covered across the EVM, Inbox, backend, runtime, and receipt suites; they are not an additional non-duplicated suite. | See the paths above and the review records. No end-to-end testnet success is claimed. |

The independent review record is **PASS** for the EVM, Soroban Inbox, and backend
tracks, with zero Critical findings and every High finding closed. That PASS is a
repository review result only. It does not prove deployed bytecode, a live
endpoint, a testnet route, real funds, or end-to-end cross-chain execution.

The recorded 18/18 Solidity result matches the current clean execution: 13
sponsor-funding cases plus 5 receipt-receiver cases. The focused backend review
totals remain review-set counts rather than a sum of every `test(...)`
declaration in the repository; the separate 45-test row is the current combined
command result. The review specifically records receipt-disappearance tests
using viem’s actual `TransactionReceiptNotFoundError`, with provider-agreement
and provider-disagreement cases. This is why the table does not add review sets
and current command results into a fabricated grand total.

The Rust package commands documented with `cargo +1.81` and `cargo +1.90` could
not be re-executed in the current application shell because its standalone
`cargo` does not include `rustup` toolchain switching. That is an environment
limitation, not a contract-test failure. The source-declared counts remain
labelled NR until the pinned toolchains produce retained execution transcripts.

## 8. Deployment and verification matrix

“Local” means source/build/test artifacts or deterministic/injected adapters.
“Verified” means independently confirmed against a deployed network artifact; a
configured address, dry-run ID, explorer link, or test fixture is not verification.

| Component | Local | Testnet | Mainnet | Verified |
|---|---|---|---|---|
| Campaign Escrow v2 | Source and local tests implemented | **No new v2 deployment; no-go for accepting funds** | None claimed | No |
| Historical Campaign Escrow v1 | Historical IDs/transactions documented | Historical v1 only | None claimed | No; IDs/transactions are documented, but no independently matched source/WASM verification is established |
| Reputation Anchor | Source and local tests | Historical ID only; source verification recorded as pending | None claimed | No; the historical ID is documented, but current source/WASM verification is not established |
| Contributor Pool (5% and 2%) | Source and local tests | No deployment IDs | None | No |
| Stellar Funding Inbox | Source, 10/10 review suite | Undeployed; official reviewed route unavailable | Undeployed | No |
| Stellar receipt adapter/OApp | Source, local tests | Undeployed | Undeployed | No |
| Arbitrum `ArbStellarSponsorFunding` | Source and local Solidity tests | No reviewed route deployment claimed | None claimed | No |
| `ReviewedOFTAdapter` / OFT | Source/wrapper and local ABI tests | No reviewed OFT deployment | None claimed | No |
| Arbitrum receipt receiver | Source and local tests | Undeployed | Undeployed | No |
| LayerZero Endpoint configuration | Deterministic/configurable boundary only | Stellar testnet Endpoint/EID not published for proposed route | Published Stellar mainnet Endpoint is documented by LayerZero, but no TogetherFi route is configured or approved | No |
| Peers | Configurable source fields only | None claimed | None claimed | No |
| EIDs | Arbitrum Sepolia `40231` is documented; no Stellar testnet EID | No Stellar testnet EID in reviewed registry | Stellar mainnet `30600` is documented; not a substitute for testnet | No TogetherFi deployment |
| Token/asset | Integer accounting and test stand-ins | Historical v1 used XLM stand-in; v2 asset unset for release | Official issuer/contract not confirmed | No |
| Campaign application infrastructure | Local APIs, ledgers, wrappers and dry-run/injected paths | No confirmed sponsor-funded v2 campaign | No production Stellar campaign claimed | No |

The historical Reputation Anchor ID is
`CBJVDSS6VUWNORYG2W7ZMHPGMCBXQN5A2CKK5OSS7L2WFR6RSFLGQILC`; the historical
Campaign Escrow v1 ID is
`CDEAOPTIFKCOFOOHNXXFLF3WAUSTXZKMIXN7NM5OM2BO3CLBX2AND34N`. No other Stellar
contract ID is asserted by this dossier.

## 9. Disclosure and public evidence package

### Safe public evidence

Provide the minimum independently useful set:

1. The independently accessible public repository and immutable source commit:
   `https://github.com/AGDAO/togetherfi-stellar/commit/0493b2cee72f52d38872152d26bf320d1281ef78`.
   It contains the current v2, Contributor Pool, Funding Inbox, receipt-adapter,
   and selected EVM LayerZero boundary source.
2. Selected source paths for v2 escrow, Contributor Pool, Reputation Anchor,
   Funding Inbox, and receipt adapter; license and build commands.
3. This accuracy statement, architecture/custody diagram, authorization matrix,
   and explicit v1-versus-v2 table.
4. Reproducible local test reports with toolchain, lockfile, commit, and
   pass/fail/ignored output.
5. The independent review summary and remediation status, clearly labeled local
   review evidence.
6. Once available, signed deployment manifest, source/WASM hashes, source
   verification pages, confirmed transaction hashes, and network labels.
7. Once available, an end-to-end testnet bundle showing source intent, LayerZero
   GUID, Inbox receipt, sponsor claim, separate native `fund`, and reconciliation.
8. Privacy/consent rules before publishing creator handles, scores, wallets, or
   campaign amounts.

Do not publish an explorer link, dry-run identifier, configured address, or
historical v1 transaction as proof of v2 or cross-chain delivery.

### Reviewer-only evidence on request

Retain signed deployment/configuration manifests, exact role/public-key inventory,
WASM/artifact hashes, independent review reports, remediation logs, test fixtures,
reconciliation exports, operational drill records, and narrowly scoped source
material needed to reproduce the claims. Redact custody locations and any
credential-bearing configuration.

### Confidential and not for publication

Private keys, seed phrases, passwords, API/RPC credentials, signer secrets,
private infrastructure URLs, customer data, internal Replit remotes, unpublished
security configuration, and proprietary business logic unrelated to verification
must remain confidential. A grant review does not require publishing the entire
application codebase.

## 10. Completed versus remaining grant-eligible work

### Already completed (not to be funded retroactively)

- Existing Arbitrum CreatorFi/SocialFi foundation and historical EVM settlement
  work.
- Historical v1 Stellar testnet Campaign Escrow and Reputation Anchor evidence.
- Current local v2 Campaign Escrow, Contributor Pool, and Reputation Anchor source.
- Local backend native Stellar wrappers, chain-isolated settlement boundaries, and
  durable receipt/intents ledgers.
- Local LayerZero receipt adapter, EVM source funding contract, reviewed wrapper,
  Funding Inbox implementation, and their repository review suites.
- Documentation of the custody boundary, economic split, route NO-GO, and
  independent local review result.

Completed means implementation/repository evidence only. It does not mean
deployed, live, or production-ready, and none should be presented as a new
retroactive funding request.

### Remaining work for candidate grant deliverables

Only the following repository-supported work belongs in a new scope:

1. Obtain an independent review of the already-implemented v2 escrow and
   Contributor Pool scope, remediate only findings produced by that review, and
   publish the review/remediation evidence. Do not rebill the implemented
   lifecycle, split, role, pause, expiry, TTL, or manifest controls.
2. Confirm the official Stellar asset and issuer, network-specific contract and
   decimals, then produce a signed immutable deployment manifest.
3. Deploy a fresh v2 testnet escrow and Contributor Pools only after authorization;
   verify source and matching WASM without overwriting historical v1 evidence.
4. Complete sponsor wallet signing, confirmation, persistence, failure handling,
   and campaign-solvency reconciliation. Activation must require confirmed native
   escrow funding.
5. Keep the optional OFT Inbox claim and native escrow `fund` as separate
   operations. Do not enable the route until an official/reviewed Stellar
   testnet path exists and the route NO-GO is re-opened successfully.
6. Remove or formally descope legacy Arbitrum-to-Stellar payout fallback/listener
   behavior from core settlement; prove Stellar failure cannot trigger Arbitrum
   payout.
7. Complete endpoint/peer/EID/asset identity verification, source verification,
   deployment receipts, and testnet end-to-end evidence.
8. Exercise pause, role rotation, unknown transaction, reorg, missing receipt,
   TTL, retry, recovery, and duplicate-operation runbooks with named owners.
9. Publish only confirmed public verification records and a privacy/consent policy.
10. Publish final reproducibility/evidence report and maintenance responsibility.

These are future acceptance artifacts, not evidence that the work is complete.

## 11. Security findings, open gates, and accepted assumptions

### Severity summary

| Severity/status | Finding |
|---|---|
| Critical | 0 recorded in the independent EVM/Soroban Inbox/backend review. |
| High | 0 open; the review record says all High findings were closed. |
| Medium closed | Financially inert receipt boundary, dependency/toolchain separation, and related controls were reviewed as PASS. |
| Medium accepted residual | The pinned Stellar surface has no receiver-side per-GUID OFT transfer hook/credit record. The configured OFT envelope’s `amount_ld` remains the delivery-amount trust root; held-liability ≤ balance, route/peer/executor checks, queue clearing, replay protection, pause, and replacement controls are defense in depth. A compromised configured OFT can still lie about the amount. |
| Low | No separate Low-finding count is recorded in the supplied review record. Do not infer zero from omission. |
| Open gate | No deployed/live Inbox, source funding contract, receipt adapter, or real cross-chain transaction evidence. |
| Open gate | No published reviewed Stellar testnet Endpoint/EID or OFT route for the proposed Arbitrum Sepolia path; route decision is NO-GO. |
| Open gate | Exact production/testnet asset, issuer, endpoints, peers, EIDs, contract IDs, bytecode/WASM, and role identities are not in a verified signed manifest. |
| Open gate | Fresh escrow deployment, source verification, independent escrow review, end-to-end testnet proof, and pause/recovery drill remain pending. |
| Open gate | Monitoring ownership, escalation rota, signer separation, and production custody procedure require an operational drill. |

“Closed” means remediation was accepted in the local review record; it does not
mean live funds exist. The security review is not an audit of the LayerZero
protocol, production key custody, or unlisted contracts. No owner may treat grant
approval as a mainnet technical gate.

## 12. Founder summary

### A. Verified facts

- The local repository contains the listed Soroban, EVM, and backend components.
- `https://github.com/AGDAO/togetherfi-stellar` and immutable source commit
  `0493b2cee72f52d38872152d26bf320d1281ef78` were independently accessible
  without authentication on 13 September 2026.
- Historical v1 Stellar IDs and transactions are documented, while current v2 and
  LayerZero route deployments are not.
- The local v2 CreatorFi split is 82/10/5/2/1 with integer base-unit accounting.
- There is no TogetherFi token and no separate TogetherFi liquidity pool.
- The proposed LayerZero testnet route is NO-GO and no real cross-chain proof exists.

### B. Things implemented

- v2 Campaign Escrow, Contributor Pools, Reputation Anchor, receipt adapter, and
  Funding Inbox source.
- Arbitrum sponsor-funding source contract and reviewed OFT wrapper.
- Arbitrum informational receipt receiver.
- Backend intent/outbox, read/reconciliation adapters, route gates, and native
  Stellar integration boundaries.

### C. Things tested

- Local Solidity route review 18/18, Soroban Inbox review 10/10, focused backend
  review 35/35, runtime adapter review 11/11, backend route/runtime review 16/16,
  and a current combined backend reproduction of 45/45.
- Source-declared tests cover escrow security, pool accounting, reputation,
  receipt authentication, replay, wrong-peer/wrong-inbox, under-delivery,
  deadline, reorg, missing receipt, pause, recovery, and chain isolation.
- Some Soroban source test files have no tracked execution transcript; those
  results are deliberately marked NR rather than inferred as passing.

### D. Things remaining to be deployed

- Fresh v2 escrow and pool contracts, current Reputation Anchor release,
  Funding Inbox, receipt adapter/receiver, EVM source funding contract, OFT
  wrapper, peers, endpoint configuration, and campaign infrastructure.
- A verified sponsor-funded testnet campaign and end-to-end route evidence.
- Mainnet artifacts are entirely future work; mainnet-only LayerZero documentation
  does not satisfy testnet proof.

### E. Things remaining to be decided

- Official settlement asset/issuer and exact network contracts/decimals.
- Signed production/testnet manifest, endpoint/EID/peer identities, roles,
  governance custody/threshold, independent review provider, and operations owner.
- Final deployment/source-verification process and route re-opening criteria.
- Any SocialFi/GameFi distribution contract and policy; the single-recipient
  CreatorFi escrow does not implement every product.

### F. Public evidence we can safely provide

Pinned source, license, selected paths, architecture/custody diagrams, redacted
test output, local review summary, historical v1 evidence with labels, and later
confirmed deployment/source-verification/transaction evidence.

### G. Private evidence we should retain

Signed manifests, exact artifact/WASM hashes, scoped reviewer material,
reconciliation exports, operational drills, role inventory, and narrowly scoped
source/fixtures needed for independent verification. Never include secrets or
private infrastructure credentials.

### H. Recommended remaining grant deliverables

Independent review and any resulting new remediation for the implemented escrow
and pool scope; reproducible builds from the public release, fresh deployment, and source
verification; asset and sponsor-wallet integration; completion of remaining
chain-isolated settlement and confirmation behavior; a reviewed, officially
supported testnet route if it becomes available; monitoring/recovery drill; and
a final public evidence pack.
Exclude retroactive v1/EVM work, campaign principal, DEX liquidity, token issuance,
and unconfirmed partner-dependent work.

### I. Unsupported claims

The repository does not support claims that v2 is deployed/live, that a LayerZero
message automatically funds or settles a campaign, that a real cross-chain
transaction or testnet route succeeded, that official Stellar testnet LayerZero
infrastructure exists for this path, that USDC or XLM is the final configured
asset, that the platform provides liquidity, that all SocialFi/GameFi flows are
implemented by this escrow, that every latest EVM contract is mainnet deployed or
audited, or that any unconfirmed organization is a partner.

## 13. Unsupported-claims checklist for grant drafting

Before submission, remove or qualify:

- “deployed,” “live,” “production-ready,” or “testnet validated” when referring to
  v2, Inbox, adapters, or the optional route;
- “bridge,” “automatic funding,” “automatic settlement,” “automatic payout,” or
  “cross-chain liquidity” unless the sentence specifically describes the
  two-step Inbox claim plus native escrow funding;
- claims of a TogetherFi token, token liquidity, DEX pool, treasury liquidity, or
  platform-funded campaign principal;
- historical v1 split/IDs as evidence of v2 policy or deployment;
- test counts that add overlapping suites or turn local review PASS into live
  end-to-end success;
- final asset/issuer, endpoint, peer, EID, deployment, source verification,
  partner, adoption, or audit claims without a matching signed artifact.

This checklist is a drafting control, not a new technical finding. The controlling
status remains: **implemented and locally tested, not deployed/live**; no real
cross-chain proof; testnet route **NO-GO**.