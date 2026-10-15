# Escrow v2 testnet validation — 4 October 2026

## Scope and current result

Escrow v2 and its two dedicated contributor-pool instances are deployed and
initialized on **Stellar Testnet**. Governance and settlement have independently
held **2-of-3** signer configurations. Their temporary setup masters have weight
zero. The supplied wallets are testnet-only; mainnet requires different owner-
provided addresses and separate authorization.

**Quorum-authorized completion is confirmed with Ayoub and King's signatures.**
The exact payout and authenticated pool credits are verified. No independent
review, mainnet deployment, or production readiness is claimed.
Ayoub is the implementation owner and sole reviewer.
Ace and King are security co-signers only.

## Contracts and asset

| Binding | Testnet address |
|---|---|
| Campaign Escrow v2 | `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ` |
| 5% MOFO contributor pool | `CAUPWNNQLQLURIJJTFW7LURJRD2OLGV4R5E6LNSUJXKSESRVGO54YP76` |
| 2% community contributor pool | `CD6TNSBSLAPXE4JXDBNTNEMMXH7DAWGUA3TR5HMP4QQIHCYM7544ED25` |
| Governance account | `GA777UZBWGPK7KPKE5BSQWH7XWDXHXQ5XLYPPZ2KSQE2WEG7OL7LKLK3` |
| Settlement account | `GDWWM5HXPY2BL4CUUJSWXPOJFGJ4RS5Y4EEVEOMJ24UB73V7QWAK5LJM` |
| Native XLM token contract | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |

The asset is **free testnet XLM, seven decimals**, solely for validation. It is
not production USDC, a final issuer decision, or a Stellar-side liquidity claim.
Sponsor, creator, referrer, treasury and eligibility accounts are isolated test
fixtures, not production custody or verified real-world campaign participants.
Individual member wallets' signer settings were not changed.

## On-chain role-account configuration

Both role accounts have low/medium/high thresholds **2/2/2**, the following
signers with weight **1** each, and their dedicated master with weight **0**:

| Holder | Testnet public signing key |
|---|---|
| Ayoub | `GDGWL4MGDMHYLDKWTKOMH6RY2B4LR3A4GDUVGXS64LKPLSDTBL4PVBLE` |
| Ace | `GD57W22FBBREDJSEN2PDRLP5FYBZQ4QHO65KN6Z6U2ZP3I2OCOALJFTH` |
| [King](https://www.linkedin.com/in/dan-dean-ver-king-ramos-56073539a) | `GARRSD6FDAUASB5UMPM6KV4UQZGWO47NBUY6F2GFC4V4UE6YOWRZDS5U` |

Only public signer addresses were used. Member secrets were never generated,
requested, read, or stored. Temporary setup keys existed only in process memory;
they cannot authorize these role accounts after handoff.

## Confirmed transaction evidence

Open a hash at `https://stellar.expert/explorer/testnet/tx/<hash>`.

| Action | Confirmed testnet transaction hash |
|---|---|
| Deploy escrow v2 | [86f5b65b…](https://stellar.expert/explorer/testnet/tx/86f5b65bfb589121aa44e207922c18ce3fcc7a0422cd7dbb68dc8c8d593b3114) |
| Deploy MOFO pool | [140b64ee…](https://stellar.expert/explorer/testnet/tx/140b64ee659de374f19c88e54dcb883ead9abe9a74aad9eb389fead3b0e088bf) |
| Deploy community pool | [e42d4894…](https://stellar.expert/explorer/testnet/tx/e42d4894f2dd0d8d46cada7ca02c46f8bfce67ed61cd658c3ed5cbe923bff486) |
| Initialize MOFO pool | [e53208a9…](https://stellar.expert/explorer/testnet/tx/e53208a969d7ae762c73e1c60aca1bd5da4743e46e8f263ba5b0714595d03774) |
| Initialize community pool | [b86f2491…](https://stellar.expert/explorer/testnet/tx/b86f2491b6e2f2b76f1fa252035f422ea730bd855246cf58aeaa8a551053bc39) |
| Initialize escrow | [7baaf300…](https://stellar.expert/explorer/testnet/tx/7baaf300372e22014248f53a382767dc02d29642c6835be5118f557c76f2ed13) |
| Handoff both roles to 2-of-3 | [9bb97971…](https://stellar.expert/explorer/testnet/tx/9bb97971a294acc396362834eefdcc1cb96c2fe8916be8e8294a4bc482451d29) |
| Fund campaign 1 with 1 test XLM | [53c2e5ed…](https://stellar.expert/explorer/testnet/tx/53c2e5ed884c8575507760640ed1d05749f22388418468ee28cafa6d674bc0e3) |
| Sponsor activation of campaign 1 | [8ecd3906…](https://stellar.expert/explorer/testnet/tx/8ecd390622067c25610596b0ce78101c5db6d956293119109b176225dc76b729) |
| Fund campaign 2 for real expiry proof | [e59cb68a…](https://stellar.expert/explorer/testnet/tx/e59cb68ac17dbd2845d2fd452a28d05cf0b7df6604849bc8d18b5872113a5e98) |
| Two-member campaign 1 completion | [ec8ca3af…](https://stellar.expert/explorer/testnet/tx/ec8ca3af04c870739b201d6746faf4fb7ded540f1cf5e8336215908803518404) |

Detailed public signed envelopes, ledger records, account configuration and
balances are in [the manifest](../evidence/testnet-escrow-v2/manifest.json).
Signed envelopes are public transaction data; no private keys are included.

## Verified code and negative checks

RPC retrieval of all three deployed WASM files matched the locally tested hashes:

- Escrow: `1cfbad51dc1932e81de857b4a22402f883584ba4c6b89388aff53929c2e6b0eb`
- Both pools: `2f24a684373cfb2683912b1a85be0b8e4ac859fb64c15b14a52050579e99a865`

Core Rust source/lockfiles were not changed by deployment. The earlier verification
remains 49 local native tests and five successful WASM builds, with two existing
historical public CI jobs passing. Eight additional local Node tests check signer validation,
thresholds, disabled-master requirements, distinct signatures and contract IDs.
The expanded public workflow is now active with all five package tests/builds
and offline Node guards. See [current CI runs](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml?query=branch%3Aescrow-v2-ci-security-evidence-20261004)
for the actual results and retained artifacts.

The disabled governance setup key's role-rotation attempt and the disabled
settlement setup key's completion attempt were rejected with `txBadAuth`.
These are **RPC ingestion rejections**, not ledger-confirmed failed transactions.
No settlement-role change or payout resulted, and recipient/escrow balances were
unchanged during the completion rejection.

An actual selected member's 1-of-3 envelope was rejected with `txBadAuth` at RPC
ingestion, not recorded as a failed ledger transaction. The same transaction body
with two independently verified member signatures was confirmed successfully.
Campaign 1 is `Completed`; exact token balance deltas are 0.82 creator, 0.10 service,
0.05 MOFO pool, 0.02 community pool, and 0.01 referrer, in test XLM.
Pool `TotalCredited` counters match the 5% and 2% transfers. Escrow liability is
exactly 1 test XLM, owed only to the still-funded expiry campaign.

Repeat completion and refund-after-completion were also checked against the
deployed contract via RPC simulation. Both returned `InvalidState`
(`Error(Contract, #6)`). These are non-mutating simulation checks, not
ledger-confirmed failed transactions; their results are retained in the manifest.

## Remaining acceptance gates

1. Member signatures, 1-of-3 RPC rejection, 2-of-3 ledger confirmation, exact payout
   deltas and liability reconciliation are complete. Duplicate terminal-operation
   protections are covered by native contract tests and deployed-contract RPC
   simulations; no extra ledger-confirmed
   duplicate-operation transaction is claimed.
2. Campaign 2 reaches its real 30-day expiry: **3 November 2026, 12:46:17 UTC**.
   Retain a confirmed refund to its original fixture sponsor then. Do not shorten
   expiry or relabel a shortened fixture as this deployed v2.
3. Owner review is the only assigned review. If the architecture team requires
   independent review, that separate condition remains unmet.
4. Expanded CI is installed using newly authorized workflow-write access;
   retain its successful run separately from on-chain acceptance evidence.
5. Final production asset, mainnet signers, operational custody and production
   launch require separate decisions; optional LayerZero components remain undeployed.

See [wallet signing instructions](testnet-signing.md).