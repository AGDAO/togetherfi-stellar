# Attested native builds and fresh USDC testnet evidence

## Confirmed build provenance, 19 October 2026

Source commit:
[`a4d5d1eb80bb000c5abb6a11d714783546707618`](https://github.com/AGDAO/togetherfi-stellar/commit/a4d5d1eb80bb000c5abb6a11d714783546707618).
The three native build/test/attestation jobs
[completed successfully](https://github.com/AGDAO/togetherfi-stellar/actions/runs/39325264354).
They use Rust 1.81.0, wasm32-unknown-unknown and the SHA-pinned compatible
StellarExpert reusable builder with Stellar CLI 22.0.1. Final artifacts contain
source/build metadata; their hashes are not the historical deployment hashes.

| Native package | Attested final WASM SHA-256 |
|---|---|
| Campaign Escrow | `6f1a3e10abd79b76cbb4101f5ea42da5e51ad0d45950093e0cacf602fd51b0ff` |
| Contributor Pool | `0842b07ff7056fcfce1df0e0586bf1779de57d4c6e4c89014a0af220d976c14f` |
| Reputation Anchor | `6e0757040318e728da1e60bcb0566ace3d02d14e456ddc25f124caa5ebe90d67` |

The final release bytes were downloaded and cryptographically verified using
GitHub CLI, enforcing repository `AGDAO/togetherfi-stellar`, the reusable signer
workflow and its exact commit `4a554f0e75c245cf4d3ea5f98f3a653d51edcd18`,
the exact source commit above and GitHub-hosted execution. Merely downloading
an artifact or trusting a manifest's `attestation_verified` boolean is insufficient;
the deployment runner reverifies the actual files.

Before creating any fresh accounts, the lifecycle runner reads the attested
WASM's own contract specification and checks every used operation's name and
positional parameter names. Escrow liability is read directly from its existing
contract-instance storage; the pool's public liability getter is not invented
on the Escrow ABI. No native source or attested bytes were modified to add it.

Release tags were explicitly bound to the source commit before publication;
they are not inferred from a default-branch tag target.

## Explorer registration, not yet a blanket verification claim

The [registration job](https://github.com/AGDAO/togetherfi-stellar/actions/runs/39325998039)
reverified all three artifacts and submitted their exact hashes and source/build
inputs from a GitHub Actions runner. StellarExpert returned HTTP 200 `{"ok":1}`
for each registration. Workspace-origin registration is rejected by the provider.

Acceptance of registration is distinct from an indexed, verified explorer result.
Confirm the new contract's explorer source status and RPC-retrieved WASM before
describing a deployed instance as source-verified.

## Fresh USDC lifecycle runner

```sh
node scripts/download-attested-native.mjs \
  a4d5d1eb80bb000c5abb6a11d714783546707618 /path/to/new-artifact-directory
node scripts/run-fresh-usdc-lifecycle.mjs --execute-testnet \
  --attested-build /path/to/new-artifact-directory/attested-build.json
```

This creates new, separate escrow and two-pool instances with the official
Circle testnet USDC SAC. It verifies every initializer binding and RPC WASM
hash, creates disposable account trustlines, and requests human-completed
free Circle faucet funding.

Current testnet storage-rent simulation quotes exceed historical 10-XLM fixture
caps (approximately 13.75 free XLM to upload and 66.20 free XLM to initialize).
Uploads remain capped at 20 free XLM; other fixture operations have a hard
400-free-XLM ceiling, below transaction uint32 limits. These are Friendbot-funded
testnet fees only, not real spending authorization or a mainnet fee policy.

Automated test signers exercise native 2-of-3 account authorization, exact
82/10/5/2/1 payout deltas, seven-decimal rounding, early-refund rejection,
completion replay rejection, pool allocation authorization and recipient claims.
These are fixture signatures, never signatures attributed to team members.
Only actual confirmed receipts and successful assertions count as completed tests.

After these checks, the dedicated role accounts hand off to the designated
Ayoub/Zyad/King public keys and disable the fixture signers. A frozen transaction
then requires two designated members' own signing-only wallet signatures.
Do not treat configuration, local auth mocks or automated fixture signatures as
that final human-member release proof.

Historical XLM deployments, member wallets and the original November expiry
fixture remain unchanged. A fresh USDC expiry fixture has its own later expiry;
the runner rejects early refunds rather than shortening the contract's 30-day
expiry to manufacture a passing receipt. Attestation is not code-review approval,
grant funding, a mainnet launch, or production acceptance. Building the Anchor
does not commit an unapproved new Anchor deployment or free grant deliverable.

## Confirmed automated USDC receipt checkpoint

The fresh run completed successfully on 19 October 2026. Circle's faucet
[delivered 20 free test USDC](https://stellar.expert/explorer/testnet/tx/78414116290321e7f6dcfd335f479feab6e52ea814551042e5bb4302cd7d5078).
No mainnet funds were used.

| New testnet instance | Address |
|---|---|
| Campaign Escrow | [`CCUYLTWIPHUMXGJNPQZZELDGJFQWXMLFW3JW3VAAYRNAMTVLADUWZS2U`](https://stellar.expert/explorer/testnet/contract/CCUYLTWIPHUMXGJNPQZZELDGJFQWXMLFW3JW3VAAYRNAMTVLADUWZS2U) |
| NFT Contributor Pool | `CDSHQH4FICZ5RFJG677UP4XTECOAO6IIJTQWB7KN2PQJXGOENEJ6BENT` |
| Revenue Contributor Pool | `CACX36IJNDCAZKXEP2DC2ZDJK4QF4R7BHFFXJTSZKJ77K74OP6HAGRQR` |

RPC-retrieved WASM hashes match the attested final files for all three instances,
and all thirteen initializer bindings passed.

- Campaign 1: [funded](https://stellar.expert/explorer/testnet/tx/23e9c12b7166cb93b73b973a54c1909943c1dcee96d8a9f733034f0747e8870f),
  [activated](https://stellar.expert/explorer/testnet/tx/09cbf87e2a7c3769d0e00be636198e8143d912a77e8756b9733ab90f6237b139),
  then [completed by two disposable signers](https://stellar.expert/explorer/testnet/tx/0546c2ca59ca749ef8ff5a2bc44040241470449921ad2953de1d9eba42649f23).
  Exact USDC deltas: creator 0.82, service 0.10, NFT pool 0.05, revenue pool 0.02,
  referrer 0.01. Escrow liability fell to zero after this completion.
- The one-disposable-signature attempt was rejected by RPC ingestion with
  `txBadAuth`, with no balance changes. The same transaction body later succeeded
  with two signatures. Transaction hashes exclude envelope signatures, so its
  successful explorer receipt is not evidence that the one-signature attempt
  was accepted. Retained signed envelopes and the RPC error distinguish them.
- Both pools accepted their dual-authority allocations, then confirmed
  recipient claims:
  [NFT claim](https://stellar.expert/explorer/testnet/tx/65593b02c485fde3321a23257b73b25bfd6d798b69f415820112222031e38f52)
  and [revenue claim](https://stellar.expert/explorer/testnet/tx/609da372ba47a8b199b8b0f17a52a333ff15c27ac1d4eab77313c2da7dd5319d).
- The [101-base-unit rounding campaign](https://stellar.expert/explorer/testnet/tx/da71215756fa14730313833d93bfc333430397efdec45028ff2b749818d4436e)
  reconciled exactly: creator 83, service 10, NFT pool 5, revenue pool 2,
  referrer 1. No base units were lost.
- Completed campaign and completed-manifest replay simulations rejected with
  the expected contract errors. Both active and unactivated early refunds
  rejected with `NotExpired`; these are simulation rejections, not ledger
  refund transactions.
- The [dedicated role-account handoff](https://stellar.expert/explorer/testnet/tx/b609409c66479088aa00cc5db971e20d828eb3dbdfef0a2d46c860b28f25e27f)
  removed the disposable signers and installed Ayoub/Zyad/King. Both accounts
  retain three weight-1 members, all thresholds 2 and disabled masters.

At this checkpoint, 1.1 test USDC remains owed by escrow: campaign 2 holds
1 test USDC for the designated-member completion, and campaign 3 holds
0.1 test USDC for its real-expiry refund fixture. This exactly matches the
RPC-read internal liability counter and escrow token balance.

### Historical pending status at 21:41 UTC

At that checkpoint the frozen campaign-2 transaction required two actual
designated members' wallet signatures. Disposable signatures above did not
satisfy that requirement.
The new USDC expiry refund and the original historical expiry proof remain
time-gated; no expiry was shortened. Final independent source reviews remain
pending.

As checked at 21:41 UTC on 19 October 2026, StellarExpert's API returned matching
WASM hashes but `validation.status = unverified` for all three new instances,
despite accepting the three registration requests. The reason for the provider's
unverified status has not been confirmed. GitHub attestations and deployed-byte
identity are proven separately; no verified-explorer claim is made.

## Designated-member USDC completion confirmed

On 19 October 2026, Ayoub and Zyad each supplied a separate public signed XDR.
Each envelope's signature was cryptographically verified against that member's
designated public key and the exact frozen transaction hash. Neither changed the
transaction body. Their two signatures were combined without reassembly or
re-simulation changes to the transaction.

The [actual member-authorized release](https://stellar.expert/explorer/testnet/tx/d985a9c5fa5b8a6c1b59f6c77d5acc5bf2f0a7657c48e0c43004eb0c5e6a6b91)
confirmed successfully in **ledger 5,284,609**.

- Native account quorum and all thirteen live initializer bindings were checked
  before submission. Both dedicated role accounts still had three weight-1
  Ayoub/Zyad/King signers, thresholds 2 and disabled masters afterward.
- Campaign 2 reached `Completed`. Exact 1-test-USDC release deltas were creator
  0.82, service 0.10, NFT pool 0.05, revenue pool 0.02 and referrer 0.01.
- Escrow token balance and its internal liability counter both fell from
  1.1 to **0.1 test USDC**. Only the separate campaign-3 expiry refund remains
  owed; no live campaign was swept or prematurely refunded.
- Pool credit counters reconciled to 0.1000005 test USDC for the NFT pool
  and 0.0400002 for the revenue pool, including the rounding fixture.
  Prior recipient claims remain 0.05 and 0.02 respectively; no claim was paid twice.

This closes the designated-member USDC release proof, not a source-review
approval or mainnet launch. Final independent reviews and the actual expired
refund proofs remain pending. At 21:51 UTC, StellarExpert still returned the
matching Escrow WASM hash with `validation.status = unverified`; its provider
verification outcome remains unresolved. No real USDC or XLM was used.
