# TogetherFi: current dossier clarification

Status checked on 31 October 2026. Documentation and evidence only; no new
features, contract redeployment, mainnet activity or real-funds authorization.

## Founder history and current engineering ownership

Ayoub (Alpha) was a co-founder of HYPEit.io and continued independently building
TogetherFi after leaving it. Ayoub is TogetherFi's founder, technical owner and
the developer writing its Soroban code.

The [later Arbitrum proposal](https://arbitrum.questbook.app/dashboard/?grantId=67d8033d46da2f90cc326ac9&chainId=10&isRenderingProposalBody=true&role=community&proposalId=6981068596f32ac6cef9994c)
documents TogetherFi.io as the later product. Historical Arbitrum deliverables
remain separate from the proposed Stellar work and are not billed twice.

## Human review and merge evidence

- Ayoub owns implementation, testing and acceptance. AI assistance is disclosed
  and does not substitute for human responsibility.
- Mohammed (@elastikchain-eth) supplied the earlier scoped PR #1 review.
- [PR #2](https://github.com/AGDAO/togetherfi-stellar/pull/2) was merged by
  @elastikchain-eth on 30 October 2026 at 16:53:58 UTC. The merged head is
  `233a2f5f6c1d15e8d9a0058642ac0fdf60e76d32`; the merge commit is
  `93d0bde62664aecd82c8c6c69ad2db8c72a93967`.
- GitHub's formal review API returned Zyad's changes-requested review and no
  separate formal approval from Mohammed. The merge is confirmed; the absence
  of a separate approval record does not establish that he did not read the code.
- [Zyad's follow-up comment](https://github.com/AGDAO/togetherfi-stellar/pull/2#issuecomment-6196327755)
  addresses `65bbc02`, not the final merged head. Do not backdate or invent
  approval of later changes.
- None of these records constitutes an independent security audit. Merge does
  not establish deployment of the merged contract bytes.

## Allocation and metrics

The proposed ask remains **$62,400**: 576 engineering hours at $95/hour
($54,720), plus 192 operations/testing hours at $40/hour ($7,680).
Ace and King each have 96 hours at $40/hour, or $3,840 each.
See [the allocation](future-work-allocation.md).

Use **585 registered users** and **at least 272 active users in the preceding
six months** as the dossier's stated baseline, not as a newly measured live count.
The requested $1,500 USDC released during the first 90 days after mainnet is a
proposed target requiring owner acceptance, not a completed metric.

## Existing evidence and unresolved items

- The public architecture page is already v4. It includes native Circle-USDC
  testnet escrow/pools, separate governance and settlement accounts and the
  confirmed designated-member USDC release. XLM evidence remains historical.
- [Attested testnet evidence](attested-usdc-testnet-evidence.md) pins deployed
  source `a4d5d1eb80bb000c5abb6a11d714783546707618`, build provenance and WASM
  hashes. Registration acceptance is not explorer source verification. The last
  recorded explorer status is unverified; do not report verification complete.
- The original XLM expiry refund becomes eligible on **3 November 2026 at
  12:46:17 UTC**. Its transaction cannot be supplied before execution is
  eligible and confirmed. The original USDC fixture expires separately on
  18 November. The accelerated 24-hour fixtures are separate test builds,
  never replacements for those original proofs.
- Zyad's co-developer classification still needs genuine development evidence
  elsewhere, or an owner-approved change to internal reviewer/co-signer with
  rebalanced hours. Existing review and signing records alone do not establish
  code authorship.
- Remaining dossier inputs: demo video URL, requested personal profile/contact
  completion, confirmation of the historical @med-amiine account attribution,
  the proposed $1,500 metric acceptance, and final-head review evidence if the
  architecture team requires it separately from the confirmed merge.
- The private Build sheet was not edited or verified through public access.
  Its nine requested rate updates must be checked in the editable sheet.

Privy and Soroswap remain future grant work. The proposed documented wallet
flow uses Privy for an authenticated session and a user-controlled Stellar
wallet through Stellar Wallets Kit/Freighter for explicit envelope signing.
It does not claim a shipped Privy-native Stellar embedded signer. Weekly
Soroswap conversion concerns platform fees only, with USDC rebates to eligible
wallets generating campaign volume, not conversion of sponsor principal.
