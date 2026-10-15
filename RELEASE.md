# Stellar-native review update

This source update is prepared for a new human review after the historical
escrow-v2 evidence PR. It is not a new contract deployment, audit, signer
rotation or mainnet release.

The public review snapshot contains only Campaign Escrow, Contributor Pool and
Reputation Anchor, with their pinned lockfiles and the required SDK host patch.
Optional bridge components and their vendored dependency trees are absent.
Native sponsor funding, refund provenance, fixed payout economics and the
deployed contract interfaces are unchanged.

See [review scope and acceptance status](docs/review-handoff.md). The intended
signer roster is Ayoub, Zyad and King; the existing on-chain roster must not be
reported as changed until the replacement is ledger-confirmed.
