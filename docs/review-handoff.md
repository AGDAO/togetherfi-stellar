# Stellar-native review handoff

## Current initialization-hardening delta

Zyad recorded a changes-requested review of the earlier snapshot. The follow-up
source binds initialization to the actual deployer and contract-ID salt, adds
strict authorization regressions and checks all deployment configuration before
funding. This changes the initialization ABI, not campaign economics or the
reputation score-record schema. It is not yet deployed. Earlier receipts and
approvals do not validate the new initializer.
See [the exact changes and remaining evidence](initialization-hardening.md).
Med Amine's final review and Zyad's follow-up on the new head remain pending.

## Review status and scope

**Final code reviews remain outstanding on
[PR #2](https://github.com/AGDAO/togetherfi-stellar/pull/2).** A review request
has been sent to Med Amine's GitHub review account,
[@elastikchain-eth](https://github.com/elastikchain-eth). Zyad's collaborator
changes-requested review is recorded; his follow-up on this patched head is pending.
Neither final approval is recorded or claimed by this handoff.
The PR must remain unmerged until both reviews and the required checks are
resolved, before the owner sends the package to The Arch team.

The public review package is limited to three native Soroban packages:
`campaign_escrow`, `contributor_pool` and `reputation_anchor`.
Campaigns use native Stellar sponsor funding, with no cross-chain funding path.

Campaign completion, original-sponsor
refunds, authenticated pool credits and the fixed 82/10/5/2/1 split are unchanged.
The 5% NFT contributor destination and 2% community destination remain separate
Stellar pool instances.

The settlement economics and pinned lockfiles are preserved. Initialization
interfaces change in the hardening delta; an escrow treasury getter is added.
The deployed reputation contract retains the historical `arbitrum_tx_hash` field
name for interface compatibility only. It is optional application metadata, not
bridge code, a funding requirement, or authenticated cross-chain evidence.
Changing its schema would require separate versioning and deployment evidence.

The Node lockfile export uses public npm registry URLs with unchanged versions
and integrity hashes. Ignore rules exclude local dependencies, secret files and
generated outputs. CI is reduced to the three retained contracts plus Node guards.

## Review ownership and prior evidence

- Ayoub is the implementation owner.
- Med Amine is the external scoped technical reviewer, not an implementation
  owner or transaction signer. His public profile is
  [@med-amiine](https://github.com/med-amiine); his PR #1 review was authored
  from [@elastikchain-eth](https://github.com/elastikchain-eth), the account
  requested for his PR #2 review.
- [Zyad](https://github.com/Zrehi) is an internal developer/peer reviewer and a confirmed testnet
  co-signer.
- Ace supports GameFi testing and operations, not contract review or the
  current Stellar signing quorum.
- King supports SocialFi testing and operations and is a
  co-signer, not a code reviewer.

## Future delivery plan

The owner-approved proposed allocation is **758 hours over 26 weeks**:
Ayoub 416, Zyad 134, Med Amine 26, Ace 91 and King 91.
See the [full allocation, schedules and responsibilities](future-work-allocation.md).
The approximately 29.2 hours/week is a combined team average, not a commitment
per reviewer. These are future delivery estimates; individual scope, availability
and compensation arrangements are confirmed separately. Historical platform work
is not included as newly completed Stellar delivery.

## Prior review evidence

The prior review remains attached to its original commit in
[PR #1](https://github.com/AGDAO/togetherfi-stellar/pull/1), under the
[@elastikchain-eth account](https://github.com/AGDAO/togetherfi-stellar/pull/1#pullrequestreview-5527032908).
The review text identifies the reviewer as Med Amine (@med-amiine).
It covered a broader source tree and must not be represented as approval of this
new snapshot or the now-confirmed signer configuration. A new review request was
sent to Med's @elastikchain-eth account against the PR #2 head. Zyad's review
request was followed by his changes-requested review. His follow-up on the
hardening commit and Med's final review remain pending. Neither approval is
claimed until each reviewer records it. A scoped technical review is not a
formal audit.

## Signer transition and fresh release confirmed

Ayoub and King authorized replacing Ace with Zyad on both dedicated governance
and settlement accounts. The transaction succeeded on 15 October 2026 in ledger
5207471, hash `acf8693c615586e35f23f98d184eb2769d8d6ed2f511c06b3d4569db69e54657`.
Both live account configurations were checked: Ayoub, Zyad and King each weight 1,
thresholds 2/2/2 and dedicated setup masters weight 0.
See [confirmed rotation evidence](../evidence/testnet-signer-rotation/confirmation.json).
Individual member wallets were not reconfigured.

The original deployed manifest and completed campaign 1 retain their historical
roster and signatures. They are not rewritten as Zyad's participation.
The original unsigned rotation request is retained for audit, not for resubmission.

Campaign 3 used one free test XLM on the same deployed escrow. Both signatures
were cryptographically verified against the exact frozen envelope. Zyad alone
was rejected at RPC ingestion with `txBadAuth`, with no payout balance changes.
Zyad and King together completed the campaign in ledger 5207647, transaction
`de497ef9bead07454b35ae4614bac21c29226279d3e1ff39f8fbd37015223f32`.
The exact 82/10/5/2/1 payout deltas, authenticated pool credits and escrow
liability reduction were verified. The original expiry campaign remains unchanged.
See [the fresh release evidence](testnet-zyad-king.md).

Med Amine has no signature task. Zyad's internal code review and transaction
signature are different roles; neither signature is an independent audit.
No signer private keys or recovery phrases should be shared.

## Quorum evidence matrix

| Members | Operation | Result |
|---|---|---|
| Ayoub + King | Replace Ace with Zyad on both role accounts | Confirmed, ledger 5207471 |
| Zyad alone | Campaign 3 completion | Correctly blocked at RPC ingestion: `txBadAuth` |
| Zyad + King | Campaign 3 completion | Confirmed, ledger 5207647; exact split verified |
| Ayoub alone | Additional campaign 4 completion | Correctly blocked at RPC ingestion: `txBadAuth`; balances unchanged |
| Zyad + Ayoub | Additional campaign 4 completion | Confirmed, ledger 5207852; exact split and accounting verified |

A valid member signature by itself does not satisfy a 2-of-3 threshold.
The single-member rejection is the expected security result, not evidence of
an invalid Zyad signature. The successful Zyad and King release proves the
new member can authorize with a second member. The
[Zyad and Ayoub test](testnet-zyad-ayoub.md) also succeeded.
Its receipt, original signed envelopes, negative test and exact
payout/accounting results are preserved. This is testnet evidence, not a grant
award, mainnet-readiness claim or substitute for the two requested code reviews.

## Items for the two reviewers

1. Native sponsor custody, immutable beneficiaries and exact payout accounting.
2. Authorization separation between governance and settlement, replay resistance
   and original-sponsor refunds.
3. Escrow-authenticated contributor credits, allocation and claim invariants.
4. The retained reputation interface and explicit absence of bridge dependencies.
5. Pinned dependencies, SDK host patch provenance and the new CI results.
6. The confirmed new-roster configuration and both fresh quorum release tests.

Reviewers should record findings and unresolved items on the new PR with the
commit they reviewed. Source review and transaction signatures are distinct.

## Remaining boundaries

The real-expiry refund fixture remains due on 3 November 2026 at 12:46:17 UTC;
this cleanup does not shorten that expiry or claim a refund has happened.
Historical test transcripts and earlier CI runs do not prove this new PR passed.
New CI must be checked separately. Source parity alone does not prove CI WASM
bytes match the previously deployed locally built binaries.

This snapshot authorizes no mainnet deployment, real-funds operations or changes
to the live TogetherFi application.
