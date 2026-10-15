# Stellar-native review handoff

## What changed after the first review

The public review repository is limited to three native Soroban packages:
`campaign_escrow`, `contributor_pool` and `reputation_anchor`.
The optional EVM/LayerZero bridge components, their package metadata, vendored
dependencies and build jobs are removed from this review snapshot. This removes
the stale foreign package identity and the unused bridge funding story rather
than disguising it with a renamed package.

Campaigns are funded natively on Stellar by the sponsor. There is no cross-chain
funding path in this source snapshot. Campaign completion, original-sponsor
refunds, authenticated pool credits and the fixed 82/10/5/2/1 split are unchanged.
The 5% NFT contributor destination and 2% community destination remain separate
Stellar pool instances. Branding cleanup does not remove or redirect either leg.

The escrow/pool executable logic, interfaces and pinned lockfiles are preserved.
The contributor-pool comment uses the generic NFT contributor allocation label.
Reputation-anchor documentation comments now describe the metadata accurately;
its executable code and interface are unchanged. In particular, the
historically deployed reputation contract retains the `arbitrum_tx_hash` field
name for interface compatibility only. It is optional application metadata, not
bridge code, a funding requirement, or authenticated cross-chain evidence.
Changing its schema would require separate versioning and deployment evidence.

The Node lockfile export uses public npm registry URLs with unchanged versions
and integrity hashes. Ignore rules exclude local dependencies, secret files and
generated outputs. CI is reduced to the three retained contracts plus Node guards.

## Review ownership and prior evidence

- Ayoub is the implementation owner.
- Med Amine is the external scoped technical reviewer, not an implementation
  owner or transaction signer. His supplied GitHub profile is
  https://github.com/med-amiine.
- [Zyad Rehioui](https://github.com/Zrehi) is an internal developer/peer reviewer and a confirmed testnet
  co-signer. King is a co-signer, not a code reviewer.

The prior review remains attached to its original commit in
[PR #1](https://github.com/AGDAO/togetherfi-stellar/pull/1).
It covered a broader source tree and must not be represented as approval of this
new snapshot or the future signer configuration. New review is requested from
Med Amine and Zyad against the exact new PR head. Neither approval is claimed
until the reviewer records it. A scoped technical review is not a formal audit.

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
new member can authorize with a second member. The owner requested the
[additional Zyad and Ayoub test](testnet-zyad-ayoub.md) for the remaining pair
also succeeded. Its receipt, original signed envelopes, negative test and exact
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

The proposed future grant workload is 758 hours over 26 weeks, approximately
29.2 aggregate hours/week. A truthful per-person breakdown is still required.
Do not assign Med Amine or Zyad hours they have not explicitly committed.
Historical overall-platform work is separate from future Stellar delivery.
This snapshot authorizes no mainnet deployment, real-funds operations or changes
to the live TogetherFi application.
