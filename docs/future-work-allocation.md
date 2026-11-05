# Proposed Stellar delivery allocation

## Planning basis

The allocation confirmed by Ayoub is **768 hours across 26 weeks**: an aggregate
average of 29.5385 hours/week, rounded to **29.5 hours/week across the whole team**.
These are proposed future hours, not a completed-work timesheet or a grant award.
The revised operations hours and rate below supersede the earlier 91-hour
allocations. Saturday is not allocated.

## Contributor allocation

| Contributor | Responsibilities | Hours |
|---|---|---:|
| Ayoub | Founder and sole developer; implementation, integration, testing and delivery coordination | 498 |
| Zyad | Internal reviewer; peer review and testing, not code authorship | 52 |
| Mohammed | Scoped contract review and follow-up, not transaction signing | 26 |
| Ace | Scoped GameFi administration, event setup, workflow testing, bug reporting, Discord support and onboarding; Facebook and Instagram operations | 96 |
| King | Scoped SocialFi administration, mission setup, workflow testing, bug reporting, Discord support and onboarding; X and Discord operations; separately designated testnet validator and 2-of-3 co-signing | 96 |
| **Total** | **Proposed future delivery, subject to eligible work packages** | **768** |

Ace and King have access to the relevant platform admin panels, **not source
code**. Neither is a contract engineer or code reviewer. Their broader platform
social-media duties do not automatically become eligible Stellar grant costs:
only documented work attributable to the agreed Stellar deliverables may be
charged to this grant. Do not bill the same work to another grant.
King is one of the three designated testnet validators and may co-sign as one
member of the 2-of-3 threshold. This transaction-signing role does not give him
source-code access or make him a developer or code reviewer.

## Compensation

| Operations contributor | Hours | Hourly rate | Proposed total |
|---|---:|---:|---:|
| Ace | 96 | $40 | $3,840 |
| King | 96 | $40 | $3,840 |
| **Combined** | **192** | **$40 per hour each** | **$7,680** |

These rates apply to the two operations/testing contributors, not the engineering
hours. The full proposed grant ask is **$62,400**: 576 engineering hours at
$95/hour ($54,720) plus 192 operations/testing hours at $40/hour ($7,680).

## Hours and evidence

- Ayoub: `498` hours over 26 weeks, averaging approximately `19.15` hours/week.
- Zyad: `2 x 26 = 52` hours for internal review and testing.
- Mohammed: `26` milestone-based review/follow-up hours.
- Ace: `96` total hours confirmed by Ayoub.
- King: `96` total hours confirmed by Ayoub.
- Combined: `498 + 52 + 26 + 96 + 96 = 768`.

Ayoub has confirmed each person's 96 hours, role and $40 rate. No additional
personal information is required to prepare the budget. The activity split below
is a proposed future work plan, not a completed timesheet or a confirmed calendar.
It replaces the earlier 91-hour cadence rather than adding unrelated hours.

| Ace: proposed Stellar-attributable outputs | Hours | Acceptance evidence |
|---|---:|---|
| Coordinate two Stellar testnet pilot sessions with GameFi community testers; prepare scenarios and approved test data | 20 | Two session plans, admin-operation checklists and attendance/outcome records |
| Execute non-code wallet/campaign workflow QA and retest reported issues | 24 | At least 12 scenario results covering onboarding, funding, receipt visibility, failed actions and retests; reproducible reports for issues actually found |
| Help participants learn testnet wallets, trustlines and USDC receipts | 20 | Onboarding guide, dated support sessions and anonymized completion/blocker log |
| Support the Stellar pilots on Discord and triage participant problems | 16 | Dated support/triage log and escalation outcomes |
| Prepare Stellar pilot education for business Facebook/Instagram channels | 12 | Six draft guidance/update items and a distribution/outcome log, subject to eligibility approval |
| Assemble milestone operations evidence and demonstrate admin workflows | 4 | Evidence index, permissions checklist and milestone walkthrough |
| **Ace total** | **96** | **$3,840 at $40/hour** |

| King: proposed Stellar-attributable outputs | Hours | Acceptance evidence |
|---|---:|---|
| Coordinate two Stellar testnet pilot sessions with SocialFi community testers; prepare scenarios and approved test data | 20 | Two session plans, mission/admin checklists and attendance/outcome records |
| Execute non-code mission/campaign workflow QA and retest reported issues | 24 | At least 12 scenario results; reproducible reports for issues actually found |
| Help participants learn testnet wallets, trustlines and USDC receipts | 20 | Onboarding guide, dated support sessions and anonymized completion/blocker log |
| Support the Stellar pilots on Discord and triage participant problems | 16 | Dated support/triage log and escalation outcomes |
| Prepare Stellar pilot education for X/Discord channels | 12 | Six draft guidance/update items and a distribution/outcome log, subject to eligibility approval |
| Participate in separately requested testnet signer drills and assemble operations evidence | 4 | Signing-only drill records and evidence index; never code-review approval or disclosure of private keys |
| **King total** | **96** | **$3,840 at $40/hour** |

GameFi/SocialFi describe the source communities and existing scoped admin roles,
not a promise to implement new sector-specific Stellar contracts or to grant
CreatorFi/code permissions. Developers perform restricted setup where necessary.
Pilot signups, bugs and adoption outcomes are not fabricated to meet an output
target. Education/social work remains subject to grant cost-eligibility review;
if excluded, reallocate within approved Stellar activities
without increasing these confirmed hour totals. Record dates, actual time and
deliverable references; do not double-charge overlapping work.

## Ownership, review and budget boundaries

Ayoub is TogetherFi's founder, sole developer and technical lead. He writes the
application and Soroban code. Review, testing, operations and transaction
co-signing are separate roles, not implementation authorship.
Zyad's 52 hours cover internal review and testing, not development. Ayoub's
498 hours cover implementation and delivery. These allocations preserve the
768-hour total and the $62,400 grant ask; they are not completed-work claims.

Profiles: [Ayoub](https://github.com/AGDAO/),
[Zyad](https://github.com/Zrehi),
[Mohammed's review account](https://github.com/elastikchain-eth),
[Ace](https://github.com/devcryptoaceph).
King is identified by his community handle in public repository materials.

The current testnet signing group is Ayoub, Zyad and King. King is one of the
three validators and can co-sign with either other signer under the 2-of-3
threshold; he cannot authorize a release alone. Ace is not in that quorum.
Reviewers are not signers merely because they review code. PR #2 merged on
30 October 2026. On 31 October, Mohammed and Zyad formally approved the exact
final head `233a2f5f6c1d15e8d9a0058642ac0fdf60e76d32`:
[Mohammed](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5691188641),
[Zyad](https://github.com/AGDAO/togetherfi-stellar/pull/2#pullrequestreview-5724217283).
The approvals are not an audit or proof of deployment.

**Reputation Anchor budget is unresolved.** Ask grant reviewers whether
new, eligible Anchor work is required and fundable. Do not promise free future
work or charge for historical work already delivered. If newly scoped work is
approved, assign a responsible contributor, hours, rate, acceptance evidence and tranche assignment;
then recalculate the total without double-counting these 768 hours.

The linked scoped review approvals are established records, not a formal audit,
mainnet launch or real-funds authorization. The proposed allocation is not a
grant award or an approved timesheet.
