# TogetherFi: current dossier clarification

Documentation and evidence updated on 4 November 2026. This update adds no app
feature, contract redeployment, mainnet activity or real-funds authorization.
The owner confirms that no new Stellar features were developed after the
requested pause. Subsequent product work was on Arbitrum, not Stellar.
Requested review fixes, existing-testnet validation and documentation are
separate from new grant-funded feature development. This is a Stellar-scope
confirmation, not a claim that the entire TogetherFi app stopped changing.

## Current engineering ownership

Ayoub (Alpha) is TogetherFi's founder, sole developer and technical owner. He
writes TogetherFi's application and Soroban code. There is no additional
technical lead or undisclosed implementation contributor to TogetherFi.
Other named contributors provide scoped review, testing, operations or
transaction co-signing; those roles do not imply code authorship.

The [later Arbitrum proposal](https://arbitrum.questbook.app/dashboard/?grantId=67d8033d46da2f90cc326ac9&chainId=10&isRenderingProposalBody=true&role=community&proposalId=6981068596f32ac6cef9994c)
documents TogetherFi.io. Historical Arbitrum deliverables
remain separate from the proposed Stellar work and are not billed twice.

## Human review and merge evidence

- Ayoub owns implementation, testing and acceptance. AI assistance is disclosed
  and does not substitute for human responsibility.
- Mohammed (@elastikchain-eth) supplied the earlier scoped PR #1 review.
- [PR #2](https://github.com/AGDAO/togetherfi-stellar/pull/2) merged on
  30 October 2026 at 16:53:58 UTC. The merged head is
  `233a2f5f6c1d15e8d9a0058642ac0fdf60e76d32`; the merge commit is
  `93d0bde62664aecd82c8c6c69ad2db8c72a93967`.
- On 31 October, GitHub recorded formal **APPROVED** reviews against that exact
  head from [Mohammed](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5691188641)
  and [Zyad](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5724217283).
- Zyad's earlier changes-requested review and follow-up comment concern earlier
  snapshots; the separate approval above is on the final merged head.
- These reviews are not an independent security audit. The merge and approvals
  do not establish deployment of the merged contract bytes.

## Allocation and metrics

The proposed ask remains **$62,400**: 576 engineering hours at $95/hour
($54,720), plus 192 operations/testing hours at $40/hour ($7,680).
Ace and King each have 96 hours at $40/hour, or $3,840 each.
See [the allocation](future-work-allocation.md).

For the updated dossier and video, use **622+ registered profiles** and
**at least 300 active users in the preceding six months**, as supplied by the
owner on 4 November 2026. These are owner-reported product-wide figures, not
Stellar mainnet adoption or a fresh independently audited activity count.
Earlier dated reporting snapshots remain historical records.
The requested $1,500 USDC release target during the first 90 days after mainnet
is owner-confirmed as a planning target, not a completed metric.

## Existing evidence and unresolved items

- The public architecture page is already v4. It includes native Circle-USDC
  testnet escrow/pools, separate governance and settlement accounts and the
  confirmed designated-member USDC release. XLM evidence remains historical.
- [Attested testnet evidence](attested-usdc-testnet-evidence.md) pins deployed
  source `a4d5d1eb80bb000c5abb6a11d714783546707618`, build provenance and WASM
  hashes. Registration acceptance is not explorer source verification. The last
  recorded explorer status is unverified; do not report verification complete.
- The original XLM 30-day refund became eligible on **3 November 2026 at
  12:46:17 UTC** and was **confirmed on 4 November 2026 at 20:22:30 UTC**:
  [transaction 34c53fb8…](https://stellar.expert/explorer/testnet/tx/34c53fb8d518c9a0f520be450c2178003c17a25a383bb779d0cb7f2a2923b5b6),
  ledger **5,560,022**. The original sponsor received exactly 1 test XLM;
  campaign status is Refunded, with zero escrow balance and liability.
  See [the full proof](original-30-day-xlm-refund.md). The original USDC fixture
  expires separately on 18 November; the accelerated 24-hour fixtures remain
  separate test builds, not replacements for either original proof.
- Zyad is an internal reviewer/tester and confirmed testnet co-signer, not an
  additional developer of TogetherFi. Reviewers will confirm his final proposed
  allocation. Existing review and signing records do not establish code authorship.
- Remaining external inputs: demo video URL and reviewer confirmation of
  Zyad's final allocation. Mohammed's public GitHub bio is now visible on
  [@elastikchain-eth](https://github.com/elastikchain-eth), checked on 4 November.
  The earlier @med-amiine attribution referred to Mohammed, whose confirmed
  review account is [@elastikchain-eth](https://github.com/elastikchain-eth);
  it does not identify a separate contributor.
- The connected Build sheet was refreshed on 4 November with the original
  refund, both final-head reviews, sole-developer ownership and the same proposed
  budget. Ace and King remain at $40/hour; no grant hours or totals were changed.
  Its Excel copy is a separate reviewer attachment, not public repository data.
- The Reputation Anchor has no public TTL-extension method. The selected design
  is external Soroban restoration/TTL-extension transactions, avoiding a
  contract ABI change. Ayoub is the owner-approved accountable payer for
  production restoration fees. The approved cadence is a weekly TTL check and
  renewal at 60 days or less remaining, targeting 3,000,000 ledgers (about 174
  days) where the target network permits; Testnet's current maximum is
  3,110,400 ledgers. The existing archived score record was restored and its
  score, instance and code TTLs were verified on Testnet.
  The mainnet fee-paying address remains undesignated; no mainnet or production
  readiness claim is authorized. See
  [the storage recovery review](initialization-hardening.md#reputation-anchor-storage-recovery-review).

Privy and Soroswap remain future grant work and are not implemented. The proposed
creator/referrer flow uses Privy `raw_sign` for explicitly authorized signing
with a user-owned embedded Stellar wallet. Freighter remains available for user
wallets and independent quorum signing. Soroban authorization payloads require
their own signatures when applicable. This matches the documented architecture
and is not a claim of a shipped Privy integration. Soroswap conversion concerns
platform fees only, with USDC rebates to eligible wallets generating campaign
volume, not conversion of sponsor principal.
