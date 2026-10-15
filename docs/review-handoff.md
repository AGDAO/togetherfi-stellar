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

The escrow/pool source, contract logic and pinned lockfiles are preserved.
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
- Zyad is an internal developer/peer reviewer and a confirmed testnet
  co-signer. King is a co-signer, not a code reviewer.

The prior review remains attached to its original commit in
[PR #1](https://github.com/AGDAO/togetherfi-stellar/pull/1).
It covered a broader source tree and must not be represented as approval of this
new snapshot or the future signer configuration. New review is requested from
Med Amine and Zyad against the exact new PR head. Neither approval is claimed
until the reviewer records it. A scoped technical review is not a formal audit.

## Signer transition confirmed; fresh release signatures pending

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

Campaign 3 is newly funded and active with one free test XLM on the same deployed
escrow. Its frozen completion envelope is prepared specifically for Zyad and King.
Their signatures and a fresh one-member rejection followed by successful
two-member completion are **pending**, not yet accepted evidence.
See [the fresh signing handoff](testnet-zyad-king.md).

Med Amine has no signature task. Zyad's internal code review and transaction
signature are different roles; neither signature is an independent audit.
No signer private keys or recovery phrases should be shared.

## Items for the two reviewers

1. Native sponsor custody, immutable beneficiaries and exact payout accounting.
2. Authorization separation between governance and settlement, replay resistance
   and original-sponsor refunds.
3. Escrow-authenticated contributor credits, allocation and claim invariants.
4. The retained reputation interface and explicit absence of bridge dependencies.
5. Pinned dependencies, SDK host patch provenance and the new CI results.
6. Once available, the actual new-roster configuration and quorum evidence.

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
