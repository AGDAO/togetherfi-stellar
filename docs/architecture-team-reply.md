# TogetherFi escrow v2 — architecture-team response

Hi team,

We have completed the core escrow v2 security changes and the requested native
Stellar Testnet funding, sponsor approval and quorum-authorized release.
The evidence and remaining acceptance gates are below. This is free-testnet
validation only, not mainnet deployment or a production-readiness claim.

## Source, tests and CI

- [Review PR](https://github.com/AGDAO/togetherfi-stellar/pull/1).
- [Tested core source commit](https://github.com/AGDAO/togetherfi-stellar/commit/5c507a3e49199ad772213ebb08667711e37a2746).
- [Passing public CI](https://github.com/AGDAO/togetherfi-stellar/actions/runs/37198273693):
  the active escrow and reputation jobs passed. The earlier incompatible
  lockfile/dependency failure was corrected.
- [Current local verification](../evidence/current/README.md): **49 native tests
  passed** across five packages, and all five pinned-toolchain WASM builds passed.
  Eight additional offline Node tests passed for signing/approval guards.
- The deployed escrow and both pool WASMs were retrieved from RPC and their hashes
  matched the tested builds. Eight deployed escrow/pool source and lockfile inputs
  matched the tested public commit. This comparison is limited to deployed core
  packages, not all five local packages.
- The [expanded CI workflow](../.github/workflows/test.yml) is now active:
  five independent contract test/WASM-build jobs plus the Node approval guards.
  [Current six-job CI results and artifacts](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml?query=branch%3Aescrow-v2-ci-security-evidence-20261004)
  replace the previous permission blocker. The optional receipt adapter's
  unused `extern crate alloc` was removed to fix its WASM allocator-link failure;
  deployed escrow/pool Rust inputs were unchanged.

## Fixed economics and committed destinations

The payout remains **82% creator / 10% treasury / 5% MOFO contributor pool /
2% community contributor pool / 1% referrer**.
`complete(campaign_id, operation_version)` accepts neither a split nor destination
arguments, and there is no split setter. Only the stored sponsor can commit
creator/referrer during activation; they cannot be replaced afterward.

[Escrow tests](../contracts/campaign_escrow/src/test.rs) exercise rejected
alternative split calls, admin-selected/replaced beneficiaries, and
governance-only settlement. Governance and settlement are separate stored roles.
The contributor pools accept one-time, escrow-authenticated campaign credits;
arbitrary token transfers do not become allocatable reward principal.

## Custody and actual quorum proof

Both dedicated governance and settlement accounts have three signer keys weighted
1 each, low/medium/high thresholds **2/2/2**, and setup master weight **0**.
Individual member wallets were not reconfigured.
The disabled setup keys' completion and governance-rotation attempts were rejected.
An actual member's one-signature completion was also rejected with `txBadAuth`
at RPC ingestion; it was not a failed ledger transaction.

The same approved transaction with Ayoub and King's verified signatures succeeded
in **ledger 5,019,829**. This demonstrates an actual two-member release, not a local
authorization mock. No member private keys or recovery phrases were collected.

## Confirmed lifecycle links

| Evidence | Stellar Testnet proof |
|---|---|
| Fresh escrow v2 deployment | [Deployment](https://stellar.expert/explorer/testnet/tx/86f5b65bfb589121aa44e207922c18ce3fcc7a0422cd7dbb68dc8c8d593b3114) |
| Fresh MOFO contributor pool | [Deployment](https://stellar.expert/explorer/testnet/tx/140b64ee659de374f19c88e54dcb883ead9abe9a74aad9eb389fead3b0e088bf) |
| Fresh community contributor pool | [Deployment](https://stellar.expert/explorer/testnet/tx/e42d4894f2dd0d8d46cada7ca02c46f8bfce67ed61cd658c3ed5cbe923bff486) |
| Governance and settlement quorum handoff | [2-of-3 configuration](https://stellar.expert/explorer/testnet/tx/9bb97971a294acc396362834eefdcc1cb96c2fe8916be8e8294a4bc482451d29) |
| Campaign 1 sponsor funding | [1 test XLM funded](https://stellar.expert/explorer/testnet/tx/53c2e5ed884c8575507760640ed1d05749f22388418468ee28cafa6d674bc0e3) |
| Sponsor approval (`activate`) | [Activation](https://stellar.expert/explorer/testnet/tx/8ecd390622067c25610596b0ce78101c5db6d956293119109b176225dc76b729) |
| Quorum-authorized release (`complete`) | [Confirmed payout](https://stellar.expert/explorer/testnet/tx/ec8ca3af04c870739b201d6746faf4fb7ded540f1cf5e8336215908803518404) |
| Funded real-expiry fixture | [Campaign 2 funding](https://stellar.expert/explorer/testnet/tx/e59cb68ac17dbd2845d2fd452a28d05cf0b7df6604849bc8d18b5872113a5e98) |

Escrow v2: `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`.

The confirmed release produced exact balance deltas of **0.82 / 0.10 / 0.05 /
0.02 / 0.01 test XLM**. Both authenticated pool-credit counters reconcile with
their transfers. Campaign 1 is completed and remaining escrow liability is exactly
**1 test XLM**, owed only to campaign 2.
Repeated completion and refund-after-completion both returned `InvalidState`
against the deployed contract in non-mutating RPC simulations. These complement
the native duplicate/terminal-state tests; they are not ledger-confirmed failures.

[Deployment evidence and complete contract/role IDs](testnet-deployment-evidence.md),
[public manifest and signed envelopes](../evidence/testnet-escrow-v2/manifest.json),
and [acceptance matrix](testnet-acceptance.md) provide the underlying records.

## Review ownership and scope

I am Ayoub Arbani, CEO/Developer, implementation owner and sole current reviewer:
[GitHub](https://github.com/AGDAO/) and
[LinkedIn](https://linkedin.com/in/ayoub-arbani/).
Most implementation was AI-assisted under my direction.
Ace and [King](https://www.linkedin.com/in/dan-dean-ver-king-ramos-56073539a)
are security co-signers, not code reviewers. Owner review and transaction signatures
are not independent human review or an external audit.

Arbitrum-triggered Stellar payments are excluded from the current grant scope.
Optional LayerZero components remain undeployed/disabled, and no live cross-chain
proof or Stellar-side USDC liquidity is claimed. Historical v1 IDs remain labeled
historical, separate from this fresh v2 deployment.
Completed implementation and validation will not be rebilled as future grant work.

## Explicit remaining gates

1. **Real timeout/refund proof:** campaign 2 reaches the unchanged 30-day expiry on
   **3 November 2026 at 12:46:17 UTC**. Its confirmed refund to the original fixture
   sponsor is still pending. We have not shortened the expiry or claimed it complete.
2. **Independent review:** no independent human escrow v2 reviewer is assigned.
   If that is required for acceptance, it remains unmet.
3. **Availability disclosure:** approximately 17 hours/week is a proposed shared
   allocation, not yet a confirmed commitment; weekly availability and parallel
   commitments still require my confirmation.
4. **Production decisions:** final settlement asset/issuer, different mainnet signer
   addresses, operational custody/drills and launch authorization remain separate
   gates. Native testnet XLM is a validation fixture, not a final asset decision.

Accordingly, the core implementation, fresh testnet deployment, fixed-split
quorum release and accounting evidence are complete. The genuine expiry refund
and the explicitly listed remaining gates are not represented as complete.

Best,
Ayoub