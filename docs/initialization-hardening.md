# Initialization hardening follow-up

## Status and scope

This source change responds to finding 1 in [Zyad's PR #2 review](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5534439255).
It is not a new deployment. Previously confirmed testnet transactions still
refer to the earlier WASM, and their receipts and signed envelopes are retained.
They must not be represented as execution of this patched initializer.

## Binding first initialization

All three native contracts now require two extra initialization arguments:

- `deployer`: the address used in the original contract-creation preimage.
- `deployment_salt`: the exact 32-byte salt used for that creation.

Before accepting any authority configuration, the contract derives its expected
address with Soroban's `with_address(...).deployed_address()` and compares it
with its own current contract address. Mismatches return `InvalidDeployment`.
It then requires authorization from that actual deployer, in addition to the
original governance/eligibility/admin authorization checks.

When the deployer also occupies one of these roles, authorization for that same
address is required once, not twice in the same Soroban invocation frame.

The salt and deployer address are public proof inputs, not secrets. Knowing the
correct inputs does not authorize initialization; the deployer must authorize
the exact initializer invocation and its configuration. An attacker cannot
substitute their own deployer address/salt without deriving a different contract
ID. This relies on the collision resistance of Stellar's contract-address
derivation and the deployer's key security.

The fix uses the existing SDK and pinned lockfiles. No new constructor support
or SDK migration is assumed. Existing initialized contracts are not upgraded,
and no historical transaction is re-signed or altered.

## Interface and deployment changes

- Escrow `initialize` gains the two proof arguments after its existing six
  configuration addresses.
- Pool `initialize` gains them after its existing four configuration addresses.
- Reputation `initialize` gains them after its admin address.
- Escrow adds a read-only `service_treasury` getter.
- Existing campaign economics, storage keys and reputation score-record fields
  remain unchanged.

The deployment script passes the correct proof for each instance and checks all
thirteen bindings before account handoff or campaign funding: escrow governance,
settlement, token, treasury and both pool destinations; plus governance,
eligibility, token and escrow address on each pool. Failed reads or mismatches
abort rather than continuing.

Future escrow deployment uses a separate `evidence/testnet-escrow-v3` directory.
The script requires explicit reviewed WASM hashes; it no longer treats the old
v2 hashes as authorization to deploy modified source.

After building and reviewing the patched WASMs, provide a local JSON file:

```json
{
  "initialization": "deployer-preimage-v1",
  "wasm_sha256": {
    "campaign_escrow": "<reviewed 64-character SHA256>",
    "contributor_pool": "<reviewed 64-character SHA256>"
  }
}
```

Only after review:

```sh
node scripts/deploy-testnet.mjs --execute-testnet --reviewed-wasm-sha256 /path/to/reviewed-build.json
```

This command is testnet-only. It is not executed by CI or ordinary tests.
Build/hash review is still required; a JSON file is not itself proof that
source and WASM match. Real deployment evidence, fresh initialization receipts
and new quorum tests remain necessary before claiming an on-chain validation of
the hardening change.

## Regression coverage

Each contract has a strict-authorization test, without `mock_all_auths`, covering
an attacker-nominated deployer, the wrong salt, correct public proof without
deployer authorization, missing configuration-authority authorization, and a
successful authorized initialization after the rejected attempts. Existing
one-time initialization coverage remains.

Node tests check proof derivation, invalid proof inputs, every individual
configuration mismatch and failed configuration reads.

## Real escrow and two-pool integration coverage

`contracts/campaign_escrow/src/integration_test.rs` registers the actual
`CampaignEscrow`, two actual `ContributorPool` instances, and the built-in
Stellar Asset Contract in one native Soroban environment. The pool crate is a
local development-only dependency, not a replacement production implementation.
All three contracts use the deployer-bound initialization interface.

There is no blanket authorization. Setup and ordinary calls authorize exact
invocations; sponsor funding includes the required nested token transfer.
Settlement only mocks the settlement account's root authorization. Escrow-to-token
and escrow-to-pool authorization must work through the real contract call graph.

The four regression cases cover:

- Rejection of direct pool credits with no authorization or authorization from
  an outsider, governance or settlement account.
- Exactly-once campaign settlement into both pools, integer rounding under the
  fixed 82% / 10% / 5% / 2% / 1% policy, and independent subsequent campaigns.
- A real second-pool credit failure caused by an issuer-authorized one-unit
  clawback. Earlier creator/service payments and the first pool's credit must
  roll back.
- A real final-token-transfer failure caused by issuer revocation of the
  referrer's token authorization. Both earlier pool credits must roll back.

The failure cases first create legitimate nonzero manifest liabilities and
partially paid allocations. Read-only snapshots compare all 13 participant and
contract token balances, escrow liability, both pools' credited/paid/liability
totals, campaign state and replay/credit markers. After the failing condition
is repaired, the unchanged settlement must succeed without manual marker resets
or duplicate payments. The failures use actual issuer calls, not mock pools or
direct mutations of escrow/pool storage.

These are native contract integration tests. They are not deployed-WASM
execution, production resource-limit measurements, new on-chain receipts,
signatures, or a formal audit. Current-head CI records the test/build result.

## Other review items

The native real escrow/two-pool integration recommendation now has the coverage
above. Dependency findings now
have a separate [usage, reachability and fix disposition](dependency-advisories.md),
including residual testnet-only Node tooling risk, not a clean-scan claim.
Formal **APPROVED** reviews by Mohammed and Zyad are recorded against PR #2's
final merged head `233a2f5f6c1d15e8d9a0058642ac0fdf60e76d32`:
[Mohammed](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5691188641)
and [Zyad](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5724217283).
These approvals are not a formal audit and do not prove deployment of the updated
source or mainnet readiness.

## Reputation Anchor storage recovery review

The Reputation Anchor has no public TTL-extension function. The reviewed
procedure is **external Soroban TTL maintenance**, not an Anchor ABI change:
adding a contract method would require a separately reviewed and deployed WASM,
while protocol-level restoration and TTL-extension transactions do not require
the contract to expose such a method.

On Protocol 23 and later, simulate an invocation that touches the archived
entry and submit the resulting restoration preamble with the invocation. Where
that automatic restoration path is unavailable, explicitly restore the required
footprint before retrying. After restoration, extend the instance, contract
code, and affected persistent score entries as needed. Verify the ledger-entry
TTLs and read the restored score records; do not reinitialize the contract or
interpret a failed read as a zero score. Track profile IDs so maintenance can
cover persistent score entries, rather than only the instance entry.
Stellar documents that TTL extension is a protocol operation and does not
require an Anchor method; extension itself is permissionless, while the
transaction's fee-paying account still needs its own signing authority. See
[State Archival](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival)
and [Storage Strategies](https://developers.stellar.org/docs/build/guides/storage/storage-strategies).

**Owner-approved operations policy (31 October 2026):** Ayoub is the authorized
accountable payer for production restoration/extension fees. Check TTLs weekly;
begin renewal when a tracked entry has 60 days or less remaining. Target
3,000,000 ledgers (about 174 days at five seconds per ledger) where the target
network's maximum permits it. Testnet's current `maxEntryTtl` is 3,110,400;
check the actual production network limit before setting a mainnet target. The
mainnet fee-paying account/address is not yet designated, so this approval does
not authorize a mainnet transaction.

## Testnet archive, restore and extension exercise

The exercise used the existing historical Anchor
`CBJVDSS6VUWNORYG2W7ZMHPGMCBXQN5A2CKK5OSS7L2WFR6RSFLGQILC`, not a new
deployment. StellarExpert records one `ScoreAnchored` event for `profile_id=1`
on 16 July 2026 (event `015624811850031105-0000`). Before restoration, the
Testnet RPC returned the persistent score entry with `lastModifiedLedgerSeq`
3,637,935 and `liveUntilLedgerSeq=0`, confirming it was archived.

| Step | Testnet evidence |
|---|---|
| Restore archived score via a simulated and submitted public `get_score(1)` invocation | [Confirmed transaction](https://stellar.expert/explorer/testnet/tx/a36a657d25642c2bdd1f02d1865de4e4a74606b2e783cd246a19a45fefcf3437), ledger 5,486,811; returned score `750`, tier `gold`, timestamp `1784206548` |
| Verify restored entry and code/instance became live | Score entry `liveUntilLedgerSeq=5,607,770` immediately after restore; subsequent read returned the same record |
| Extend score entry, Anchor instance, and WASM code externally | [Confirmed TTL-extension transaction](https://stellar.expert/explorer/testnet/tx/a33c8b96337845837ec35e8596d08a52c7c0ec969a3c1a632a34e4055fdbf189), ledger 5,486,856 |
| Verify extension | All three entries have `liveUntilLedgerSeq=8,486,856`, exactly 3,000,000 ledgers beyond the extension ledger; the score remained `750` / `gold` |

The network was Stellar Testnet, Protocol 29. The extension transaction used
`ExtendFootprintTTLOp` with a read-only footprint containing `Score(1)`, the
contract instance and WASM code. The restore transaction paid 1.4139099 testnet
XLM; the extension transaction paid 33.5838852 testnet XLM. Both transactions
used one-use Friendbot-funded Testnet accounts generated in memory: restore
fee source `GDQJ7XRD3MTSIV32TWGBLU4BKQ2DPTLNMO5ANLF2ZHRNFQCSPOAQGKFU`; extension
fee source `GDI4XQDTTB45FDLP7FBHROVP45VHSIJ4DYYCDCBR2KEEW4G6YXUJYTJ4`. No Ayoub
wallet, member secret or mainnet account was used. No private key was retained.

An initial attempt to extend exactly to Testnet's configured maximum of
3,110,400 ledgers returned `EXTEND_FOOTPRINT_TTL_MALFORMED`; a later read
confirmed the score TTL had not changed. The RPC failure response reported
34.8712272 testnet XLM charged, but its transaction hash was not retained, so
this failed attempt is recorded as a diagnostic, not successful evidence. The
confirmed 3,000,000-ledger target keeps headroom below that boundary. The
successful restore and extension are confirmed network evidence; local Soroban
tests are not being substituted for them.

**Mainnet gate:** this exercise validates the recovery mechanics only. It does
not authorize mainnet deployment, production fee spending, or a readiness
claim. The owner-designated production fee account/address and any separate
deployment authorization remain prerequisites.
