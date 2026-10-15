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

## Other review items

This delta does not silently declare the dependency advisories or real
escrow-to-pool integration-test recommendation resolved. Their disposition and
Med Amine's final review remain pending. No formal audit or mainnet readiness
is claimed.

For storage operations, persistent/instance TTL expiry means archival rather
than deletion. The operator must monitor TTL and pay extension/restoration fees.
Clients must simulate transactions and include required restores on supported
protocols, or explicitly restore before retrying. A failed read must not be
treated as zero balance or permission to reinitialize. End-to-end restoration
exercises and assignment of the production fee-paying operator remain pending.
