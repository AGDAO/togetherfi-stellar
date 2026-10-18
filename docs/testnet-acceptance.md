# Escrow v2 testnet acceptance: quorum completion confirmed

V2 and both contributor pools are deployed and initialized on testnet. Role-account
configuration, disabled setup-key rejections and quorum completion are verified.
Real expiry refund is not yet claimed. Separate internal human peer review by
Zyad (INKY) is owner-confirmed, not an external audit. Historical v1 IDs
remain historical only. See [current evidence](testnet-deployment-evidence.md).

The owner permits the architecture team's requested testnet validation. This is
testnet-only authorization, not mainnet or real-funds approval. Supplied signing
wallets are strictly testnet-only; mainnet requires different owner-provided wallets.

## Selected signer policy

Use a **2-of-3 approval policy**, implemented through native
Stellar account signer weights and thresholds.
The current on-chain signer holders are **Ayoub, Zyad, and King**.
Both dedicated role accounts were confirmed rotated in ledger 5207471 on
15 October 2026. The original deployment evidence retains Ayoub, Ace and King.
The [fresh Zyad and King release test](testnet-zyad-king.md) completed in ledger
5207647 after a Zyad-only RPC rejection. Exact payout deltas and pool credits
were verified; this is separate from the historical completion evidence.
Zyad is also a developer/peer reviewer; King is a co-signer, not a code reviewer.
Mohammed Idmoussi (Med Amine) is a technical reviewer, not a signer. All three members must have independently
held Stellar signing keys; none may be a placeholder or an extra backend-held key.

Use three independently held Stellar signing keys, proposed weight 1 each and
required quorum 2. Protect governance as well as settlement. Signer names and
Non-Stellar addresses are not Stellar G-account public keys and cannot configure
this quorum. No private keys should be provided in chat.

## Required decisions

- Retain the owner's testnet-only authorization and identify the exact validation
  deployment in its manifest. Do not treat this as mainnet authorization.
- Validation uses free native testnet XLM with seven decimals. The production
  asset/issuer remains undecided; this fixture is not a USDC liquidity claim.
- Name governance and settlement account custodians; choose independent settlement
  signers, weights, and quorum. A backend-controlled quorum does not solve single-key
  custody. Check the applicable Stellar account thresholds and Soroban authorization,
  including signer/threshold changes and recovery paths.
  Governance must also require independent authorization: a backend-only governance
  key could otherwise propose a new backend-controlled settlement address and accept
  that role using its own new key. Test insufficient-quorum role rotation as well as
  insufficient-quorum settlement. No backend-held key combination may control either
  required quorum, account reconfiguration, or recovery by itself.
- Record Ayoub's owner review and the owner-confirmed separate human peer review
  by Zyad (INKY), another team developer. Internal peer review is not an
  external audit; no separate review report is attached here. Do not nominate Ace
  or King as code reviewers or treat transaction signatures as review evidence.

## Required evidence

The historical baseline below is supplemented by the current-roster
[Zyad and King release](testnet-zyad-king.md), ledger 5207647, and
[Zyad and Ayoub release](testnet-zyad-ayoub.md), ledger 5207852.
Both had a valid single-member envelope blocked with `txBadAuth`, followed by
confirmed two-member completion and exact payout/accounting reconciliation.
These confirmations do not close the real-expiry refund gate below.

| Evidence | Current status |
|---|---|
| Exact source commit, lockfile hashes, compiler/target, fresh WASM hashes | Local inputs retained; all three RPC-retrieved WASMs match tested hashes |
| Successful CI run for exact deployed source | Historical escrow/reputation CI passed; the current review workflow checks the three retained native contracts plus Node approval guards. See [current PR checks](https://github.com/AGDAO/togetherfi-stellar/pull/2/checks). CI success does not establish byte-for-byte equality with the older locally built deployed WASMs |
| Fresh contributor-pool instances and escrow v2 contract ID | Deployed; IDs and confirmed transactions in evidence |
| Initialized governance/settlement/pool/token bindings | Initialized and checked |
| Settlement signer weights and thresholds observed on-chain | Both roles: 1/1/1 weights, 2/2/2 thresholds, setup master 0 |
| Backend-only / insufficient-signature completion rejected; balances unchanged | Disabled setup key and actual member 1-of-3 rejected at RPC ingestion with txBadAuth; no payout |
| Quorum-authorized completion succeeds with fixed 82/10/5/2/1 payout | Confirmed in ledger 5019829 with Ayoub and King signatures; exact deltas verified |
| Sponsor funding and sponsor activation (approval) | Campaign 1 funded with 1 test XLM and activated; confirmed hashes recorded |
| Admin alternative split and destination attempts rejected | Local tests/CI; retain any testnet rejection evidence |
| Expiry/timeout boundary and refund to original sponsor | Campaign 2 funded; real expiry 3 November 2026 12:46:17 UTC; refund pending |
| No double completion/refund and campaign liability reconciled | Native tests and deployed-contract RPC simulations reject repeat completion/refund after completion with InvalidState; not failed ledger transactions. Remaining escrow liability exactly 1 test XLM |
| README v2 ID and StellarExpert links for each accepted transaction | Deploy/init/handoff/fund/activate/complete links published; real expiry refund pending |

Map the team's `approve` to sponsor-authorized `activate`, `release` to `complete`,
and `timeout` to the expiry condition exercised by `refund_expired`. There is no
separate `approve`, `release`, or `timeout` entry point. Preserve the production
30-day expiry rather than changing it just to produce an immediate timeout proof.
Plan the actual expiry evidence window or agree an explicitly separate test fixture
with the team; a shortened fixture must not be described as the deployed v2 code.
The additional 30-day live-ledger refund proof is deferred technical validation,
not an established grant-payment prerequisite. No direct architecture-team or
Stellar instruction requiring that specific wait is cited in this checklist.
Confirm actual acceptance criteria rather than treating this document as proof
of an externally imposed funding condition.

Record network, contract IDs, transaction hashes, ledger times, sponsor/recipient
balances, and StellarExpert testnet links. Failed simulation/local mock output is
not a confirmed on-chain transaction. Publish no private keys or signer secrets.