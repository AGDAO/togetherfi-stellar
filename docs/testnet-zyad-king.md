# Fresh Zyad and King testnet release

The signer replacement is confirmed in ledger 5207471. Campaign 3 is a new
one-free-test-XLM campaign on the existing deployed escrow, not a redeployment.
Historical campaign 1 and the real-expiry campaign 2 are unchanged.

**Status:** campaign 3 is active. Zyad and King's wallet signatures, the
one-signature rejection and the two-signature release are still pending.

## Frozen request

- [Unsigned XDR](../evidence/testnet-zyad-king/complete-campaign-3.xdr.txt).
- [Public manifest and funding receipts](../evidence/testnet-zyad-king/manifest.json).
- Hash: `de497ef9bead07454b35ae4614bac21c29226279d3e1ff39f8fbd37015223f32`.
- Signature deadline: 16 October 2026, 10:47:24 UTC (12:47:24 Berlin).
- Network passphrase: `Test SDF Network ; September 2015`.
- Source: dedicated settlement account
  `GDWWM5HXPY2BL4CUUJSWXPOJFGJ4RS5Y4EEVEOMJ24UB73V7QWAK5LJM`.
- One operation: invoke `complete(3, 1)` on escrow
  `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`.
- Campaign principal: 1 free test XLM. Fixed split: 82/10/5/2/1.
- Maximum fee: 66.7053738 free test XLM, paid by the dedicated fixture-funded
  settlement account, not the members' personal wallets. It includes bounded
  resource/rent headroom; it is not the payout or a mainnet fee quote.

## Each member signs the same original file separately

1. Select their own signer account and Test Net in Freighter.
2. Connect through https://play.freighter.app/#/extension/playground/requestAccess.
3. Open https://play.freighter.app/#/extension/playground/signTransaction.
4. Paste the complete unsigned XDR above in Enter transaction XDR.
5. Paste the exact testnet passphrase, including spaces around the semicolon.
6. In Request signature from specific public key enter their own address:
   - Zyad: `GDTAV2MA7TX4ATKBPEF4G7SFT7GHPIRZ4UVMQKNISPEGC5AE4P3AWWG7`
   - King: `GARRSD6FDAUASB5UMPM6KV4UQZGWO47NBUY6F2GFC4V4UE6YOWRZDS5U`
7. Click Sign Transaction XDR, inspect the testnet completion and approve.
8. Return the entire Result text or signedTxXdr as a public text file.

Do not submit, re-simulate, rebuild, or sign the old signer-replacement XDR.
No private keys or recovery phrases are requested. The two public signed
envelopes must be verified against the frozen hash before combining.

After collecting both signatures, first preserve and submit a one-member
envelope to record the expected RPC bad-auth rejection. Then preserve and submit
the matching two-member envelope, verify terminal state and exact balance deltas,
and attach the actual results to PR #2. Neither step is claimed complete yet.
These transaction signatures do not constitute code-review approvals.
