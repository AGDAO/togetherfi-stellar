# Stellar-native dependency advisory disposition

Assessment date: 15 October 2026. Scope: the three native packages
`campaign_escrow`, `contributor_pool`, `reputation_anchor`, and this directory's
Node validation package. Optional bridge packages, their lockfiles, the
application's dependencies and the vendored host's standalone development
lockfile are not part of this review. The patched host as resolved by each
native lockfile **is** included.

## Decision and approval boundary

This is a source-level advisory disposition, not a clean vulnerability scan,
formal audit, deployment authorization or final reviewer approval. No dependency,
contract, host patch or deployed bytecode is changed here. Keep the SDK 21
protocol/toolchain baseline rather than force a major SDK migration to silence
alerts. Findings below are either mitigated by existing build settings, lack
the triggering usage, are inactive lockfile entries, or have an explicitly
bounded tooling risk. None is silently suppressed in the scanner.

The Node HTTP risk is retained for **trusted-operator, testnet-only validation**,
not accepted for a public service or real-funds operator. This recommendation
and its assumptions still require Mohammed Idmoussi and Zyad's review
of the actual PR
head. Neither their acceptance of these dispositions nor their final approval
is claimed. Reassess before production, after any relevant code/feature/profile
change, when endpoints or inputs become untrusted, or within 30 days of this
assessment, whichever comes first.

Historical RPC-verified WASM hashes and signed transactions cover only their
original bytes. The initialization-hardening source is already different from
that deployment. This document does not retrospectively prove historical build
flags or reachability in those binaries. Any future dependency change requires
fresh locked tests, builds, hashes and source-to-binary review; old receipts do
not validate changed builds. Network validators run their own Soroban host,
not the test host bundled here.

## Inputs and reproducibility

SHA-256 of the assessed lockfiles:

| Input | SHA-256 |
|---|---|
| `contracts/campaign_escrow/Cargo.lock` | `dfe3d9e25f044270770f0f301e3b3be1f8e51515bd4243c3e6eda90cd0976201` |
| `contracts/contributor_pool/Cargo.lock` | `d1a00c6a94980e0b574b0d39a21454f82d86f453a245d4b8a3f3f1cde9b06633` |
| `contracts/reputation_anchor/Cargo.lock` | `568a7a0a6273f7f631709cf71c2de103a87804c2e62b6945ab984e92a1792978` |
| `package-lock.json`, public registry normalization | `bc5e15725fce91d62738faf15e3d1ce004ebcbb7fad587f9773d7be953728a7e` |

The Node hash uses `publicLockfile` from `scripts/review-package.mjs`, which
normalizes registry URLs only; package versions and integrity remain unchanged.
The pool and anchor locks have 178 packages; the escrow lock has 179 because
the real-pool integration tests include the first-party contributor-pool package.
All have the same affected third-party versions listed below.
Although manifests say `soroban-sdk = "21.7.6"` (a compatible range), all three
lockfiles actually resolve SDK and SDK macros **21.7.7**.

**19 October 2026 input reconciliation:** the escrow hash previously printed in
this table was stale. The current local escrow lock is byte-identical to the
public PR head's lock. Its external registry package records (versions, sources,
checksums and dependency edges) were compared with the still-assessed pool
lock and are identical; its additional first-party pool is the actual source
included in this review. The patched host record is compared separately as a
local dependency. No lockfile or dependency was changed to satisfy the guard.
This correction is not a fresh advisory-feed scan or an approval of the pool's
source. Existing reachability constraints, known tooling risks and pending human
review remain applicable.

The dependency scanner was run on this workspace and matched to exact
name/version pairs in these lockfiles, not treated as a Stellar-only scanner.
Its workspace-wide totals include unrelated application and bridge packages
and are not the native result. The native matches are nine advisory identifiers
across seven packages; the two `time` identifiers describe the same issue.
There is one high, six moderate and one low distinct underlying native issue.
`paste` is a maintenance warning, not a demonstrated exploit.
No critical native match was returned. SAST and privacy/dataflow scans were not
requested or run; no whole-application security conclusion follows.

From this package directory:

```sh
npm audit --package-lock-only --json
for c in campaign_escrow contributor_pool reputation_anchor; do
  cargo tree --locked --manifest-path contracts/$c/Cargo.toml \
    --target wasm32-unknown-unknown --edges normal,build
  cargo tree --locked --manifest-path contracts/$c/Cargo.toml --edges normal,build,dev
done
npm test
```

The npm audit returned three high affected-package records: SDK (a propagated
parent warning), Axios and TOML. There are 36 Axios and two TOML advisory records,
not three independent high exploits. Advisory feeds can change; preserve this
assessment's date and rerun rather than assuming later counts will match.
Target-specific locked dependency trees were inspected for all three packages.
No Rust build or fresh on-chain validation is claimed by this assessment.

Verification of this documentation/guard delta: `npm test` passed all 26
offline tests, including exact-lock identity, release overflow settings,
conservative API-use sentinels and inclusion in the public review export.
The existing workspace SDK was used; a clean `npm ci`, network transport
regression or new Rust/WASM build was not performed. These static sentinels
require reassessment when tripped; they do not prove general absence of every
indirect vulnerable call or replace human source review.

## Contract guest and code-generation exposure

| Package / advisory | Affected usage and reachability | Disposition and fix |
|---|---|---|
| `soroban-sdk-macros 21.7.7`, high, [GHSA-4chv-4c6w-w254](https://github.com/advisories/GHSA-4chv-4c6w-w254) / CVE-2026-26267 | A `#[contractimpl] impl Trait for Contract` can dispatch to a same-named inherent function instead, bypassing checks. This is a **build-time macro capable of changing exported WASM behavior**, not safely dismissed as host-only. The three production `lib.rs` files use inherent `#[contractimpl] impl Contract` blocks, not trait exports. | Trigger absent in assessed source. Keep this restriction under regression checks. Feed reports a patched branch at 22.0.10; advisory also names 25.1.1. Neither is an in-place SDK 21 patch. Do not independently override the tightly coupled macros. Reassess any trait export or SDK migration. |
| `soroban-sdk 21.7.7`, moderate, [GHSA-96xm-fv9w-pf3f](https://github.com/advisories/GHSA-96xm-fv9w-pf3f) / CVE-2026-24889 | Guest `Bytes::slice`, `Vec::slice` and `Prng::gen_range::<u64>` can wrap extreme bounds with overflow checks disabled. No calls to these APIs occur in the three contract implementations. All three release profiles explicitly enable `overflow-checks = true`, the upstream workaround. | Mitigated by both current usage and build profile. Preserve overflow checks, including build-environment overrides. Patched branches include 22.0.9, 23.5.1 and 25.0.2; no blind major migration. Any override disabling checks invalidates this disposition. |
| `soroban-sdk 21.7.7`, moderate, [GHSA-x2hw-px52-wp4m](https://github.com/advisories/GHSA-x2hw-px52-wp4m) / CVE-2026-32322 | Incorrect unreduced BN254/BLS12-381 `Fr` equality can affect cryptographic validation. These contracts accept addresses, amounts, identifiers and score data, not curve scalars, and use none of the affected curve APIs. | Trigger absent; a broad version-range match does not establish these newer curve APIs are present/used in this SDK baseline. Patched branches include 22.0.11, 23.5.3 and 25.3.0. Reassess before adding curve verification. |
| `stellar-xdr 21.2.0`, moderate, [GHSA-x57h-xx53-v53w](https://github.com/advisories/GHSA-x57h-xx53-v53w) / CVE-2026-29795 | `StringM::from_str` skips the declared maximum length check. XDR is present in host proc-macro/spec generation **and** the guest dependency graph; do not label the entire crate test-only. The vulnerable `FromStr` implementation is gated by `alloc`; the guest XDR path is no-alloc, while build-host XDR has alloc/serde. No native implementation or patched host source directly invokes `StringM::from_str` on untrusted text. | No identified triggering path in assessed contracts. Build inputs/specs must remain trusted. Use length-checked `TryFrom` if adding textual `StringM` conversion. Upstream fix is 25.0.1; SDK 21's exact XDR 21.2.0 coupling prevents an independent compatible major replacement. |

## Host, tests and inactive lock entries

| Package / advisory | Affected usage and reachability | Disposition and fix |
|---|---|---|
| Patched `soroban-env-host 21.2.1`, low, [GHSA-pm4j-7r4q-ccg8](https://github.com/advisories/GHSA-pm4j-7r4q-ccg8) | Host storage-conversion state can remain set after a failed conversion, breaking a subsequent muxed-address event/serialization after a handled subcall failure. The SDK selects this host for non-WASM targets; WASM selects `soroban-env-guest`. The source does not use muxed addresses or handle a failed call then emit one. | Retain as a bounded local test-host limitation, not proof about the network host. Whole-transaction rollback is the upstream described outcome, not corrupted state. Fix is host/protocol 26, not a compatible drop-in for SDK 21. Do not represent the existing BLS host patch as fixing this advisory. |
| `serde_with 3.16.1`, moderate, [GHSA-7gcf-g7xr-8hxj](https://github.com/advisories/GHSA-7gcf-g7xr-8hxj) | Empty entries passed to the `KeyValueMap` serializer can panic. Reached as a dependency of host-side XDR/spec macros and ledger snapshots; it is not a guest runtime serde serializer. No `KeyValueMap` usage was found in the native source, patched host or the SDK/XDR source inspected. | Trigger absent in current trusted build/test inputs. 3.21.0 is semver-compatible with XDR's 3.x requirement but requires Rust 1.88, incompatible with the pinned Rust 1.81 WASM build. Do not force it or silently upgrade the compiler. Adding arbitrary host-side serializers invalidates this disposition. |
| `time 0.3.44`, moderate, [RUSTSEC-2026-0009](https://rustsec.org/advisories/RUSTSEC-2026-0009.html), alias [GHSA-r6v5-fh4h-64xc](https://github.com/advisories/GHSA-r6v5-fh4h-64xc) / CVE-2026-25727 | RFC2822 parsing can exhaust the stack. Present in the lock via `serde_with`'s optional time support, but absent from the active current-host tree and all three WASM normal/build trees. Native code does not parse RFC2822. | Inactive lock entry, not linked guest exposure. Do not delete lock entries by hand. 0.3.47 fixes it but requires Rust 1.88 despite being a patch release. Reassess enabling serde time support or a new target; do not waive it for arbitrary host tooling. |
| `paste 1.0.15`, moderate maintenance warning, [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) | Unmaintained proc-macro through `wasmi_core -> soroban-wasmi -> soroban-env-host/common` on the test-host tree. Not present in the guest normal/build tree. | Retain for the SDK-locked test VM; no patched `paste` release exists. Replacing with `pastey` or another macro requires changing upstream VM dependency/source, not merely choosing a newer compatible version. Reassess with a separately tested SDK/VM upgrade; this is maintenance risk, not a claimed reachable vulnerability. |

The cached registry index confirms Rust 1.88 for `time 0.3.47` and
`serde_with 3.21.0`. A semver-compatible version alone is not proof of compiler,
protocol, generated spec, resource budget or deployed-byte compatibility.

## Node validation tooling

The locked SDK is `@stellar/stellar-sdk 15.1.0`, pulling **exactly**
`axios 1.15.0` and `toml ^3.0.0` (locked 3.0.0). Node modules are not compiled
into Soroban WASM. They still matter to transaction construction and operator
network requests; they are not harmless merely because they are tooling.

`scripts/testnet-common.mjs` constructs RPC/Horizon clients with literal
`https://soroban-testnet.stellar.org` and `https://horizon-testnet.stellar.org`.
Other fetches use fixed testnet Horizon/Friendbot origins. Inputs are local
reviewed fixtures and signed envelopes. There is no public request handler,
user-selected RPC origin, TOML discovery, federation or WebAuth flow here.

**Axios, high:** network request construction is reachable. Prototype-pollution
gadgets need polluted objects/process state; the scripts do not merge untrusted
HTTP config or TOML into request configuration. Multipart/blob, browser cookie/
XSRF, form-data recursion, streamed uploads and explicit HTTP/2 transports are
not used. Proxy, redirect, response-size and endpoint-compromise risks cannot
be proved absent just from a literal starting URL. In particular a hostile
redirect/response or credential-bearing environment proxy can expose the Node
adapter findings. Retention is justified only for the narrow testnet execution
boundary above, not as non-reachability of all Axios advisories.

Required operating limits for that retention: run an isolated process with no
untrusted plugins/input/config objects; no mainnet keys, production API tokens
or authenticated proxy credentials; use only the reviewed testnet endpoints and
fixtures; stop on unexpected redirects or network behavior. Do not expose these
scripts as a service. These are operating requirements, not claims that Axios
currently enforces redirect or response bounds.

**TOML, high:** [GHSA-82x6-q7mm-w9cf](https://github.com/advisories/GHSA-82x6-q7mm-w9cf)
(uncontrolled recursion, fixed 4.2.0) and
[GHSA-v5mp-jgw5-2x6j](https://github.com/advisories/GHSA-v5mp-jgw5-2x6j)
(prototype pollution, fixed 4.1.2). The SDK's TOML discovery surface is not
called by these scripts. Loading the SDK is not itself parsing TOML. Retain
only while this remains true; allowing untrusted TOML would also invalidate
the Axios pollution assumptions.

**Compatible fix analysis:** a refresh within the SDK's declared dependency
ranges cannot fix its exact Axios pin or TOML 3.x range. `npm audit` suggests
SDK 15.0.0, a downgrade, not a validated fix; do not run `audit fix --force`.
An Axios 1.20.0 override stays within Axios major 1 but violates the SDK's exact
pin and needs explicit SDK transport regression testing. TOML 4.2+ is outside
the parent range and must not be forced blindly. The current upstream SDK
17.2.1 instead declares Axios 1.20.0 and `smol-toml ^1.6.1`, but also requires
Node >=22.12.0 (this package allows >=20) and changes major dependencies/APIs.
That is a deliberate tooling migration requiring fixture/XDR, signature,
simulation, fee, and transport tests, not a compatible patch to apply here.
No such upgrade, installation or validation is claimed.

### Axios advisory inventory

All entries below apply to locked 1.15.0 and share the bounded tooling
disposition above. IDs link via `https://github.com/advisories/<ID>`.

| IDs | Trigger family | Severity |
|---|---|---|
| GHSA-pf86-5x62-jrwf, GHSA-6chq-wfr3-2hj9, GHSA-q8qp-cvcw-x6jj, GHSA-3g43-6gmg-66jw, GHSA-35jp-ww65-95wh | Prototype-pollution request/credential/config gadgets | high |
| GHSA-w9j2-pvgh-6h63, GHSA-3w6x-2g7m-8v23, GHSA-898c-q2cr-xwhg, GHSA-mmx7-hfxf-jppx, GHSA-7q8q-rj6j-mhjq, GHSA-vh66-26gq-q6x8, GHSA-9fr6-4gfg-395g, GHSA-j8rh-479h-cp32 | Prototype-pollution status, response, method and header gadgets | moderate |
| GHSA-pmwg-cvhr-8vh7, GHSA-pjwm-pj3p-43mv | Proxy exclusions / local-address SSRF | high |
| GHSA-m7pr-hjqh-92cm, GHSA-f4gw-2p7v-4548, GHSA-44g4-m2mj-wpvx | Proxy exclusions / IP aliases / CIDR | moderate |
| GHSA-p92q-9vqr-4j8v, GHSA-j5f8-grm9-p9fc | Proxy credentials leaking on redirects | high |
| GHSA-mghh-pgcx-3jjj | Redirect/proxy host-normalization ReDoS | high |
| GHSA-3pq3-5fj3-cg6v, GHSA-542g-h47m-68v8 | HTTP/2 DNS/proxy bypass and unhandled error | high |
| GHSA-hfxv-24rg-xrqf, GHSA-777c-7fjr-54vf | Cookie-name ReDoS / unbounded resources | high |
| GHSA-445q-vr5w-6q77, GHSA-4hqw-qxg8-jxx2, GHSA-xx6v-rp6x-q39c | Multipart/header injection, browser XSRF | moderate |
| GHSA-5c9x-8gcm-mpgx, GHSA-vf2m-468p-8v99, GHSA-mwf2-3pr3-8698, GHSA-jqh4-m9w3-8hp9 | Streamed upload/response size limits | moderate |
| GHSA-62hf-57xw-28j9, GHSA-pmv8-rq9r-6j72, GHSA-42h9-826w-cgv3 | Form serialization recursion | moderate |
| GHSA-xhjh-pmcv-23jw | URL parameter null-byte injection | low |

This inventory is not a permanent allowlist. Before expanding tooling use or
handling real funds, remove the residual Node findings with a supported SDK/
Node combination or complete explicit transport overrides and regression tests,
then obtain reviewer acceptance of the changed package.
