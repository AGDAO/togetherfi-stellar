# Confirmed Stellar testnet signer replacement

**Confirmed:** Ayoub and King's signatures authorized the replacement in
ledger 5207471 on 15 October 2026, transaction
[`acf8693c615586e35f23f98d184eb2769d8d6ed2f511c06b3d4569db69e54657`](https://stellar.expert/explorer/testnet/tx/acf8693c615586e35f23f98d184eb2769d8d6ed2f511c06b3d4569db69e54657).
Both dedicated accounts now have Ayoub, Zyad and King at 2-of-3.
See [confirmation and account observations](../evidence/testnet-signer-rotation/confirmation.json).
The charged fee was 400 stroops (0.00004 free test XLM); 0.004 was the fee cap.
Do not sign or resubmit the already-consumed replacement envelope.

The original request and instructions below are historical audit material.
The separate [Zyad and King release test](testnet-zyad-king.md) is now confirmed
in ledger 5207647. Neither consumed envelope needs further signatures.
The [additional Zyad and Ayoub release](testnet-zyad-ayoub.md) completed in
ledger 5207852; it is also consumed and needs no further signing.

The owner requested replacing Ace with Zyad on the dedicated governance and
settlement accounts. The intended roster is Ayoub, Zyad and King, with weight 1
each, thresholds 2/2/2 and the dedicated setup masters remaining weight 0.
Med Amine is a reviewer, not a signer.

**The unsigned request alone was not evidence of an account change.** Preserve the
original deployed manifest and historical validation evidence. Do not describe
Zyad as an active on-chain signer until the replacement is ledger-confirmed.
No contract redeployment, mainnet action, payment, or member-wallet configuration
change is requested.

## Transaction to inspect

See [the request](../evidence/testnet-signer-rotation/request.json) for its frozen
hash, exact deadline, public account observations and before/after signer maps.
See [the unsigned envelope](../evidence/testnet-signer-rotation/replace-ace-with-zyad.xdr.txt).

- Network: Stellar Testnet, passphrase `Test SDF Network ; September 2015`.
- Transaction source: dedicated governance account
  `GA777UZBWGPK7KPKE5BSQWH7XWDXHXQ5XLYPPZ2KSQE2WEG7OL7LKLK3`.
- Other operation source: dedicated settlement account
  `GDWWM5HXPY2BL4CUUJSWXPOJFGJ4RS5Y4EEVEOMJ24UB73V7QWAK5LJM`.
- Exactly four `setOptions` operations: add Zyad with weight 1 and remove Ace
  with weight 0, on each of those two accounts.
- Zyad public key:
  `GDTAV2MA7TX4ATKBPEF4G7SFT7GHPIRZ4UVMQKNISPEGC5AE4P3AWWG7`.
- Removed Ace public key:
  `GD57W22FBBREDJSEN2PDRLP5FYBZQ4QHO65KN6Z6U2ZP3I2OCOALJFTH`.
- No threshold, master-weight, destination, contract, or payment changes.
- Fee cap: **0.004 free test XLM**, paid by governance, not the individual members.

## Required signatures

Ayoub and King can authorize the replacement together because they are existing
signers of both accounts. Zyad cannot authorize his own addition; his signature
is not part of the current authorized quorum.

Each existing member should use their own wallet on Test Net. Connect to the
official [Freighter signing-only playground](https://play.freighter.app/#/extension/playground/requestAccess),
then use [signTransaction](https://play.freighter.app/#/extension/playground/signTransaction)
with the exact passphrase above and the overriding network-name option unset.
Inspect all four operations before signing.

Return each public signed XDR envelope. Do not send private keys, recovery phrases
or wallet access. Do not submit it yourself, re-simulate it, edit it, or sign the
archived campaign-1 completion envelope. Both signatures must cover the same
frozen hash. If the deadline expires or source sequence changes, prepare a new
request and collect fresh signatures; old signatures cannot be reused.

After receiving the two signatures, verify their common transaction body and
signatures before submitting to testnet. Retain ledger confirmation, then verify
the new signer roster, unchanged thresholds and disabled masters on **both**
accounts. Only then update current manifests and repeat acceptance validation.
Historical campaign-1 signatures do not prove Zyad's new signer participation.
