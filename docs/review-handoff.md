# Stellar-native review handoff

## Current initialization-hardening delta

Zyad first recorded a changes-requested review of an earlier snapshot. The follow-up
source binds initialization to the actual deployer and contract-ID salt, adds
strict authorization regressions and checks all deployment configuration before
funding. This changes the initialization ABI, not campaign economics or the
reputation score-record schema. It is not yet deployed. The formal approvals
below apply to the reviewed merged head; historical receipts do not validate
execution of the new initializer.
See [the exact changes and remaining evidence](initialization-hardening.md).
Mohammed and Zyad later submitted formal approvals on the final merged head.

The [dependency advisory disposition](dependency-advisories.md) now separates
guest/code-generation, local host/test and Node-tooling exposure against the
exact native lockfiles. It retains bounded testnet-only tooling risk and does
not claim zero advisories, approval or validation of changed deployed bytes.

## Review status and scope

**PR #2 is merged** at `93d0bde62664aecd82c8c6c69ad2db8c72a93967`.
GitHub records two formal **APPROVED** reviews against final head
`233a2f5f6c1d15e8d9a0058642ac0fdf60e76d32`:
[Mohammed](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5691188641)
and [Zyad](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5724217283).
These reviews are not a formal security audit and do not prove that the changed
source has been deployed.

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

- Ayoub is TogetherFi's founder, sole developer and technical owner.
- Mohammed is the external scoped technical reviewer, not
  an implementation owner or transaction signer. His PR #1 review was authored
  from [@elastikchain-eth](https://github.com/elastikchain-eth), the account
  requested for his PR #2 review.
- [Zyad](https://github.com/Zrehi) is an internal peer reviewer/tester and a
  confirmed testnet co-signer, not an additional implementation author.
- Ace supports GameFi testing and operations, not contract review or the
  current Stellar signing quorum.
- King supports SocialFi testing and operations and is one of the three
  designated testnet validators. He can co-sign as one member of the 2-of-3
  quorum, but is not a developer or code reviewer and has no source-code access.

## Future delivery plan

The owner-approved proposed allocation is **768 hours over 26 weeks**:
Ayoub 416, Zyad 134 proposed, Mohammed 26, Ace 96 and King 96.
See the [full allocation, schedules and responsibilities](future-work-allocation.md).
The approximately 29.5 hours/week is a combined team average, not a commitment
per reviewer. These are future delivery estimates; individual scope, availability
and compensation arrangements are confirmed separately. Historical platform work
is not included as newly completed Stellar delivery.
Ace and King each have a proposed rate of $40/hour ($3,840 each; $7,680 combined).
They administer scoped GameFi/SocialFi panels and support operations, not source
code. King is also a validator who may co-sign with another signer under the
2-of-3 threshold; he cannot release funds alone. Broader social-media work is
not automatically a Stellar grant expense. The updated grant ask is **$62,400**:
576 engineering hours at $95/hour ($54,720) and 192 operations/testing hours at
$40/hour ($7,680).
Any additional Reputation Anchor budget and acceptance scope remain to be agreed.

## Prior review evidence

The prior review remains attached to its original commit in
[PR #1](https://github.com/AGDAO/togetherfi-stellar/pull/1), under the
[@elastikchain-eth account](https://github.com/AGDAO/togetherfi-stellar/pull/1#pullrequestreview-5527032908).
The review text identifies the reviewer as Mohammed.
It covered a broader source tree and must not be represented as approval of this
new snapshot or the now-confirmed signer configuration. A new review request was
sent to Mohammed's @elastikchain-eth account against the PR #2 head.
Both Mohammed and Zyad formally approved the final source head
`233a2f5f6c1d15e8d9a0058642ac0fdf60e76d32`, using the review links above.
Zyad's earlier changes-requested snapshot does not override that final-head
approval. These scoped reviews are not a formal audit or deployment proof.

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

Mohammed has no signature task. Zyad's internal code review and transaction
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

The original 30-day XLM fixture became eligible on 3 November 2026 at 12:46:17 UTC.
Its [refund confirmed](original-30-day-xlm-refund.md) on 4 November 2026 at
20:22:30 UTC, ledger 5,560,022. The sponsor received exactly 1 test XLM;
campaign status is Refunded and escrow balance and liability are zero.
No expiry or reviewed contract source was changed.
Historical test transcripts and earlier CI runs do not prove this new PR passed.
New CI must be checked separately. Source parity alone does not prove CI WASM
bytes match the previously deployed locally built binaries.

This snapshot authorizes no mainnet deployment, real-funds operations or changes
to the live TogetherFi application.
