# Escrow v2 testnet acceptance — pending authorization

No command in this checklist has been executed as a deployment. No v2 ID,
multisig configuration, explorer transaction, or independent review is claimed.
Historical v1 IDs and transactions remain historical only.

## Required decisions

- Confirm grant approval and obtain separate owner authorization before deployment.
  Resolve the architecture team's requested pre-grant sequencing with the owner.
- Select the exact testnet asset/issuer/contract and seven-decimal base units.
- Name governance and settlement account custodians; choose independent settlement
  signers, weights, and quorum. A backend-controlled quorum does not solve single-key
  custody. Check the applicable Stellar account thresholds and Soroban authorization,
  including signer/threshold changes and recovery paths.
- Name the independent PR reviewer and record review of escrow v2 and configuration.

## Required evidence

| Evidence | Current status |
|---|---|
| Exact source commit, lockfile hashes, compiler/target, fresh WASM hashes | Generate in CI; bind to eventual deployment |
| Successful CI run for exact deployed source | Pending |
| Fresh contributor-pool instances and escrow v2 contract ID | Not deployed |
| Initialized governance/settlement/pool/token bindings | Not configured |
| Settlement signer weights and thresholds observed on-chain | Not configured |
| Backend-only / insufficient-signature completion rejected; balances unchanged | Not exercised |
| Quorum-authorized completion succeeds with fixed 82/10/5/2/1 payout | Not exercised |
| Sponsor funding and sponsor activation (approval) | Not exercised |
| Admin alternative split and destination attempts rejected | Local tests/CI; retain any testnet rejection evidence |
| Expiry/timeout boundary and refund to original sponsor | Not exercised |
| No double completion/refund and campaign liability reconciled | Not exercised |
| README v2 ID and StellarExpert links for each accepted transaction | Pending deployment |

Map the team's `approve` to sponsor-authorized `activate`, `release` to `complete`,
and `timeout` to the expiry condition exercised by `refund_expired`. There is no
separate `approve`, `release`, or `timeout` entry point. Preserve the production
30-day expiry rather than changing it just to produce an immediate timeout proof.
Plan the actual expiry evidence window or agree an explicitly separate test fixture
with the team; a shortened fixture must not be described as the deployed v2 code.

Record network, contract IDs, transaction hashes, ledger times, sponsor/recipient
balances, and StellarExpert testnet links. Failed simulation/local mock output is
not a confirmed on-chain transaction. Publish no private keys or signer secrets.