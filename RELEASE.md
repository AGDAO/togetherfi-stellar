# Stellar-native final review candidate

## Follow-up: initialization hardening

The current source binds each initializer to its actual deployer and deployment
salt and requires deployer authorization in addition to the original authority
checks. The deployment script verifies thirteen configuration bindings before
handoff or funding. Strict native regressions and deployment-helper tests cover
the new guards. The initialization ABI changes, and a treasury getter is added.
No dependency versions, payout economics or score-record schema change.

This follow-up is not deployed. Historical testnet evidence remains intact and
does not prove execution of this new source. Final reviews remain pending.
See [initialization hardening](docs/initialization-hardening.md).

Dependency warnings have a separate
[advisory disposition](docs/dependency-advisories.md), with exact lockfile
identities, reachability, compiler/protocol compatibility constraints and
remaining testnet-only Node HTTP risk. Dependencies and historical WASM evidence
are unchanged. Final reviewers must assess these dispositions separately.

This package awaits final code reviews on PR #2 before handoff to The Arch team.
The review request for Mohammed Idmoussi was sent to
[@elastikchain-eth](https://github.com/elastikchain-eth), the account that
authored his PR #1 review. Zyad's collaborator changes-requested review is
recorded; his follow-up on the patched head is pending.
No final approval or merge is claimed.
It is not a new contract deployment, audit or mainnet release.

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
No fresh human code-review approval is claimed.

The additional Zyad and Ayoub release completed campaign 4 in ledger 5207852
using one free test XLM on the same escrow. Ayoub's valid signature alone was
correctly blocked with `txBadAuth`; together their signatures authorized the
exact 82/10/5/2/1 split. Actual fee: 0.1999542 free test XLM.
The earlier completed campaign 3 and original expiry fixture remain unchanged.
See [campaign 4 confirmation and public envelopes](docs/testnet-zyad-ayoub.md).

The [future delivery allocation](docs/future-work-allocation.md) is 768 hours over
26 weeks: Ayoub 416, Zyad 134, Mohammed Idmoussi 26, Ace 96 and King 96. This is a
combined team average of approximately 29.5 hours/week, not per reviewer.
Ace and King are scoped platform administrators and operations contributors,
not developers or code reviewers. Their proposed compensation is $20/hour:
$1,920 each and $3,840 combined. Reputation Anchor budget eligibility remains
to be agreed; no free future delivery or completed-work payment is implied.
The original real-expiry refund validation remains scheduled separately.
