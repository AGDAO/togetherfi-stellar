# Additional Zyad and Ayoub testnet quorum test

**Pending:** campaign 4 is funded and active using one free test XLM.
Zyad and Ayoub must sign the new frozen envelope below. No release, signature
verification or insufficient-quorum result is claimed yet for campaign 4.
The successful Zyad and King campaign 3 remains unchanged.

## Why this additional pair

Ayoub and King authorized the signer replacement in ledger 5207471.
Zyad and King authorized campaign 3 completion in ledger 5207647.
This additional campaign tests Zyad and Ayoub as the remaining pair.
The three member keys each have weight 1; both dedicated accounts require
weight 2 at all threshold levels, with setup masters disabled.

A single valid authorized signature is deliberately insufficient. An expected
`txBadAuth` ingestion rejection demonstrates threshold enforcement, not a bad
member signature. After both new signatures are verified, first test Ayoub
alone and confirm balances unchanged, then submit the two-member envelope.
Record actual results before changing this document from pending.

## Frozen request

- [Unsigned campaign 4 XDR](../evidence/testnet-zyad-ayoub/complete-campaign-4.xdr.txt).
- [Public manifest and funding receipts](../evidence/testnet-zyad-ayoub/manifest.json).
- Hash: `74fb8d534a7e5c704d9d15f9a5097da309253c6a599e03ede85a657b841c51fd`.
- Signature deadline: 16 October 2026 at 11:01:23 UTC (13:01:23 Berlin).
- Network passphrase: `Test SDF Network ; September 2015`.
- Source: dedicated settlement account
  `GDWWM5HXPY2BL4CUUJSWXPOJFGJ4RS5Y4EEVEOMJ24UB73V7QWAK5LJM`.
- One operation: `complete(4, 1)` on existing escrow
  `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`.
- Principal: 1 free test XLM. Fixed split: 82/10/5/2/1.
- Fee cap: 0.4455308 free test XLM from the dedicated free-funded settlement
  account, not the individual members' personal wallets.

## Collect the two signatures

Each member selects their own account on Test Net, connects through
https://play.freighter.app/#/extension/playground/requestAccess, then opens
https://play.freighter.app/#/extension/playground/signTransaction.
Both paste the same original unsigned campaign 4 XDR and the exact passphrase.
In Request signature from specific public key enter their own address:

- Ayoub: `GDGWL4MGDMHYLDKWTKOMH6RY2B4LR3A4GDUVGXS64LKPLSDTBL4PVBLE`
- Zyad: `GDTAV2MA7TX4ATKBPEF4G7SFT7GHPIRZ4UVMQKNISPEGC5AE4P3AWWG7`

Click Sign Transaction XDR and inspect the completion before approving.
Return each complete Result text separately for cryptographic verification.
Do not submit, re-simulate or rebuild it. Old rotation/campaign 3 signatures
cannot authorize campaign 4. No private keys or recovery phrases are requested.
Transaction signatures are separate from code-review approvals or a formal audit.
