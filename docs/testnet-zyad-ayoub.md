# Additional Zyad and Ayoub testnet quorum test

**Confirmed:** Zyad and Ayoub completed campaign 4 using one free test XLM in
ledger **5207852**, 15 October 2026 at 11:14:07 UTC.
[Successful transaction](https://stellar.expert/explorer/testnet/tx/74fb8d534a7e5c704d9d15f9a5097da309253c6a599e03ede85a657b841c51fd).
Both signatures were verified against the exact unchanged frozen transaction.
**Do not sign or resubmit this consumed envelope.** The instructions below are
historical signing records, not a new request.

## Why this additional pair

Ayoub and King authorized the signer replacement in ledger 5207471.
Zyad and King authorized campaign 3 completion in ledger 5207647.
This additional campaign tests Zyad and Ayoub as the remaining pair.
The three member keys each have weight 1; both dedicated accounts require
weight 2 at all threshold levels, with setup masters disabled.

A single valid authorized signature is deliberately insufficient. An expected
`txBadAuth` ingestion rejection demonstrates threshold enforcement, not a bad
member signature. Ayoub's valid single-signature envelope was correctly blocked
with `txBadAuth` at RPC ingestion; no balances or accounting counters changed.
The same transaction body with both valid member signatures then succeeded.
The rejected attempt is not a failed confirmed ledger transaction. Its hash
matches the later success because envelope signatures are not part of the hash.

## Reconciled results

The exact base-unit deltas were creator +8,200,000, service +1,000,000,
NFT pool +500,000, community pool +200,000, referrer +100,000,
and escrow -10,000,000, matching the fixed 82/10/5/2/1 split of one test XLM.
Escrow liability decreased by 10,000,000 to 10,000,000, which covers the original
expiry fixture. NFT/community authenticated cumulative credits increased to
1,500,000/600,000; pool TotalPaid counters remained unchanged at zero.

Actual charged fee: **1,999,542 stroops = 0.1999542 free test XLM**, below the
signed cap of 0.4455308. The original campaign 2 and completed campaign 3 were
compared in full and unchanged. No real funds, mainnet or member-wallet
configuration changes were involved.

## Consumed frozen request and public evidence

- [Original unsigned archive, do not sign](../evidence/testnet-zyad-ayoub/complete-campaign-4.xdr.txt).
- [Public manifest, rejection and ledger receipt](../evidence/testnet-zyad-ayoub/manifest.json).
- [Zyad's signed envelope](../evidence/testnet-zyad-ayoub/zyad-signed.xdr.txt).
- [Ayoub's signed envelope](../evidence/testnet-zyad-ayoub/ayoub-signed.xdr.txt).
- [Confirmed combined envelope](../evidence/testnet-zyad-ayoub/quorum-signed.xdr.txt).
- Hash: `74fb8d534a7e5c704d9d15f9a5097da309253c6a599e03ede85a657b841c51fd`.
- Historical signature deadline: 16 October 2026 at 11:01:23 UTC (13:01:23 Berlin).
- Network passphrase: `Test SDF Network ; September 2015`.
- Source: dedicated settlement account
  `GDWWM5HXPY2BL4CUUJSWXPOJFGJ4RS5Y4EEVEOMJ24UB73V7QWAK5LJM`.
- One operation: `complete(4, 1)` on existing escrow
  `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`.
- Principal: 1 free test XLM. Fixed split: 82/10/5/2/1.
- Fee cap: 0.4455308 free test XLM from the dedicated free-funded settlement
  account, not the individual members' personal wallets.

## How the two signatures were collected (historical only)

Each member selected their own account on Test Net, connected through
https://play.freighter.app/#/extension/playground/requestAccess, then opens
https://play.freighter.app/#/extension/playground/signTransaction.
Both pasted the same original unsigned campaign 4 XDR and the exact passphrase.
In Request signature from specific public key each entered their own address:

- Ayoub: `GDGWL4MGDMHYLDKWTKOMH6RY2B4LR3A4GDUVGXS64LKPLSDTBL4PVBLE`
- Zyad: `GDTAV2MA7TX4ATKBPEF4G7SFT7GHPIRZ4UVMQKNISPEGC5AE4P3AWWG7`

They used Sign Transaction XDR and returned each complete Result separately
for cryptographic verification. Neither re-simulated nor rebuilt the envelope.
The verified signatures were combined, persisted, and submitted on testnet.
Old rotation/campaign 3 signatures cannot authorize campaign 4. No private
keys or recovery phrases were requested or used by the backend.
Transaction signatures are separate from code-review approvals or a formal audit.
