# Current local verification

Verified on 4 October 2026 using the compatible committed lockfiles. These are
local working-tree transcripts, not deployment or independent human-review evidence.
Each package directory contains `verification.txt` and `WASM.sha256`.

Deployment and actual quorum proof are separate:
[testnet manifest](../testnet-escrow-v2/manifest.json).
Source matching to the tested public commit was verified for the eight deployed
escrow/pool source and lockfile inputs. Do not generalize that comparison to all
five packages: the optional receipt-adapter working-tree source differs from
that public candidate. Its transcript is local-only evidence, and it is undeployed.

| Package | Native tests | Compiler | WASM target |
|---|---:|---|---|
| Campaign Escrow | 15 passed | 1.81.0 | wasm32-unknown-unknown |
| Contributor Pool | 8 passed | 1.81.0 | wasm32-unknown-unknown |
| Reputation Anchor | 11 passed | 1.81.0 | wasm32-unknown-unknown |
| Receipt Adapter | 5 passed | 1.90.0 | wasm32v1-none |
| Funding Inbox | 10 passed | 1.90.0 | wasm32v1-none |

Total: **49 passed**, no failures; all five locked release WASM builds passed.

Public candidate:
https://github.com/AGDAO/togetherfi-stellar/commit/5c507a3e49199ad772213ebb08667711e37a2746

Public PR:
https://github.com/AGDAO/togetherfi-stellar/pull/1

Public CI:
https://github.com/AGDAO/togetherfi-stellar/actions/runs/37198273693

The historical public run passed two jobs (escrow and reputation); it is not
five-package CI evidence. The expanded workflow is now active on the review
branch and includes five contract test/WASM-build jobs plus the Node guards.
[Current results and CI-generated artifacts](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml?query=branch%3Aescrow-v2-ci-security-evidence-20261004)
are distinct from these retained local transcripts. The receipt-adapter source
was aligned with the locally validated version by removing unused `extern crate alloc`.
No immutable release tag has been changed.