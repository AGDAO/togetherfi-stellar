# Stellar-native final review candidate

This package awaits final code reviews by Zyad Rehioui and Med Amine on PR #2
before handoff to The Arch team. No final approval or merge is claimed.
It is not a new contract deployment, audit or mainnet release.

The public review snapshot contains only Campaign Escrow, Contributor Pool and
Reputation Anchor, with their pinned lockfiles and the required SDK host patch.
Optional bridge components and their vendored dependency trees are absent.
Native sponsor funding, refund provenance, fixed payout economics and the
deployed contract interfaces are unchanged.

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

The [future delivery allocation](docs/future-work-allocation.md) is 758 hours over
26 weeks: Ayoub 416, Zyad 134, Med Amine 26, Ace 91 and King 91. This is a
combined team average of approximately 29.2 hours/week, not per reviewer.
The original real-expiry refund validation remains scheduled separately.
