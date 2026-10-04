# Escrow v2 testnet acceptance — authorized, configuration pending

No command in this checklist has been executed as a deployment. No v2 ID,
multisig configuration, explorer transaction, or independent review is claimed.
Historical v1 IDs and transactions remain historical only.

The owner permits the architecture team's requested testnet validation. This is
testnet-only authorization, not mainnet or real-funds approval. Deployment remains
blocked on the actual public signer/account configuration, not on a new request
for grant approval.

## Selected signer policy

Reuse the Arbitrum model's **2-of-3 approval policy**, implemented through native
Stellar account signer weights and thresholds rather than an EVM contract.
Ayoub Arbani and Draco are the first two proposed signer holders. The third
holder is not selected yet and must actually be provisioned; it is not a
placeholder or a third key held by the backend.

Use three independently held Stellar signing keys, proposed weight 1 each and
required quorum 2. Protect governance as well as settlement. Signer names and
Arbitrum addresses are not Stellar G-account public keys and cannot configure
this quorum. No private keys should be provided in chat.

## Required decisions

- Retain the owner's testnet-only authorization and identify the exact validation
  deployment in its manifest. Do not treat this as mainnet authorization.
- Select the exact testnet asset/issuer/contract and seven-decimal base units.
- Name governance and settlement account custodians; choose independent settlement
  signers, weights, and quorum. A backend-controlled quorum does not solve single-key
  custody. Check the applicable Stellar account thresholds and Soroban authorization,
  including signer/threshold changes and recovery paths.
  Governance must also require independent authorization: a backend-only governance
  key could otherwise propose a new backend-controlled settlement address and accept
  that role using its own new key. Test insufficient-quorum role rotation as well as
  insufficient-quorum settlement. No backend-held key combination may control either
  required quorum, account reconfiguration, or recovery by itself.
- Name the independent PR reviewer and record review of escrow v2 and configuration.
  Draco's proposed signer role is not evidence that he independently reviewed the
  PR. Confirm whether he authored the Soroban changes; a teammate can review a PR
  independently of its author, but do not represent that as an external audit.

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