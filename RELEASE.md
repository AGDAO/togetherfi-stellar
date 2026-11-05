# Stellar-native reviewed source snapshot

## Follow-up: initialization hardening

The current source binds each initializer to its actual deployer and deployment
salt and requires deployer authorization in addition to the original authority
checks. The deployment script verifies thirteen configuration bindings before
handoff or funding. Strict native regressions and deployment-helper tests cover
the new guards. The initialization ABI changes, and a treasury getter is added.
No dependency versions, payout economics or score-record schema change.

This follow-up is not deployed. Historical testnet evidence remains intact and
does not prove execution of this new source. PR #2 merged on 30 October 2026;
on 31 October, Mohammed and Zyad submitted formal **APPROVED** reviews against
the exact final head `233a2f5f6c1d15e8d9a0058642ac0fdf60e76d32`:
[Mohammed's review](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5691188641)
and [Zyad's review](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5724217283).
Those approvals are review evidence, not deployment evidence or a formal audit.
See [initialization hardening](docs/initialization-hardening.md).

Dependency warnings have a separate
[advisory disposition](docs/dependency-advisories.md), with exact lockfile
identities, reachability, compiler/protocol compatibility constraints and
remaining testnet-only Node HTTP risk. Dependencies and historical WASM evidence
are unchanged. The review approvals are documented below; the disclosed
testnet-only tooling risk and audit boundary remain unchanged.

The merged PR #2 review snapshot and its approval records are documented above.
The approvals apply to the reviewed source head; they do not establish a new
contract deployment, audit or mainnet release.

The public review snapshot contains only Campaign Escrow, Contributor Pool and
Reputation Anchor, with their pinned lockfiles and the required SDK host patch.
Optional bridge components and their vendored dependency trees are absent.
Native sponsor funding, refund provenance, fixed payout economics and the
historically deployed contract instances remain unchanged. The current source's
initialization interface changes as described above.

See [review scope and acceptance status](docs/review-handoff.md). Both dedicated
testnet accounts now have Ayoub, Zyad and King at 2-of-3. The fresh campaign 3
release by Zyad and King is confirmed in ledger 5207647 after a Zyad-only
`txBadAuth` rejection. The payout split and accounting were verified.
These signing receipts are separate from the formal PR #2 approvals linked above.

The additional Zyad and Ayoub release completed campaign 4 in ledger 5207852
using one free test XLM on the same escrow. Ayoub's valid signature alone was
correctly blocked with `txBadAuth`; together their signatures authorized the
exact 82/10/5/2/1 split. Actual fee: 0.1999542 free test XLM.
At that historical checkpoint, completed campaign 3 and the original expiry
fixture were unchanged. The original Campaign 2 refund subsequently confirmed
on 4 November 2026, as recorded below.
See [campaign 4 confirmation and public envelopes](docs/testnet-zyad-ayoub.md).

The [future delivery allocation](docs/future-work-allocation.md) is 768 hours over
26 weeks: Ayoub 498, Zyad 52, Mohammed 26, Ace 96 and King 96. This is a
combined team average of approximately 29.5 hours/week, not per reviewer. The
updated grant ask is **$62,400**: 576 engineering hours at $95/hour ($54,720)
and 192 operations/testing hours at $40/hour ($7,680).
Ace and King are scoped platform administrators and operations contributors,
not developers or code reviewers, and have no source-code access. Their proposed
compensation is $40/hour: $3,840 each and $7,680 combined. King is also one of
the three designated testnet validators and may co-sign a release with another
signer under the 2-of-3 threshold; this is signing authority, not development or
code-review access. Reputation Anchor budget eligibility remains to be agreed;
no free future delivery or completed-work payment is implied.
Ayoub is TogetherFi's founder and sole developer. Zyad's 52 hours are for
internal review and testing, not implementation authorship. The revised
individual allocations preserve the 768-hour total and the $62,400 grant ask.

The original 30-day XLM refund [confirmed on 4 November 2026 at 20:22:30 UTC](docs/original-30-day-xlm-refund.md),
ledger 5,560,022, after eligibility began on 3 November at 12:46:17 UTC.
The original sponsor received 1 test XLM; escrow balance and liability are zero.
