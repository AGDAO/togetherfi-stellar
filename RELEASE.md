# Stellar-native review update

This source update is prepared for a new human review after the historical
escrow-v2 evidence PR. It is not a new contract deployment, audit or mainnet release.
The separate native testnet signer replacement is confirmed in ledger 5207471.

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

An additional Zyad and Ayoub release is prepared as campaign 4 using one free
test XLM on the same escrow. Their new signatures and its acceptance results
are pending. Completed campaign 3 evidence is preserved separately.
