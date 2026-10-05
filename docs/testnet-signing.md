# Sign the escrow v2 testnet completion

**Campaign 1 is now completed.** These are archived signing instructions for the
confirmed transaction, not a request for more signatures. Do not submit it again.

This is a testnet-only security acceptance transaction. It releases **1 free test
XLM** from campaign 1 to isolated test recipients using **82/10/5/2/1**. It does
not change signer thresholds, rotate roles, approve a production asset, or move
real funds. Signatures are transaction approvals, not code-review endorsements.

## Check the transaction

- Network: **Stellar Testnet**.
- Source: settlement multisig
  `GDWWM5HXPY2BL4CUUJSWXPOJFGJ4RS5Y4EEVEOMJ24UB73V7QWAK5LJM`.
  This is deliberately different from your individual signing wallet.
- Contract: `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`.
- Exactly one operation: `complete`, campaign ID **1**, operation version **1**.
- Expected transaction hash:
  `ec8ca3af04c870739b201d6746faf4fb7ded540f1cf5e8336215908803518404`.
- Signing deadline: **5 October 2026, 13:45:55 UTC**
  (**15:45:55 Berlin time**). If expired, request a refreshed envelope;
  all signers must sign the new version rather than combining different envelopes.

## Two signatures, collected in sequence

1. Open [the unsigned XDR file](../evidence/testnet-escrow-v2/complete-campaign-1.xdr.txt)
   and copy its full contents.
2. Open [Freighter's official signing-only playground](https://play.freighter.app/#/extension/playground/signTransaction).
   First run [requestAccess](https://play.freighter.app/#/extension/playground/requestAccess)
   and approve connecting your individual wallet. Keep Freighter on **Test Net**.
   Paste the XDR into the transaction input, leave the `network` option unset,
   and set `networkPassphrase` to exactly `Test SDF Network ; September 2015`.
3. Run **signTransaction** and approve in Freighter. Use your own testnet wallet
   and verify the details above. Do not use Stellar Lab's simulation/import flow,
   import the multisig source account, or enter any secret keys.
4. Copy the returned **signedTxXdr** and send it to Ace or King.
5. That second signer pastes the **first signer's signed XDR**, not the original
   unsigned file, into the same signing-only playground and signs with their own
   Freighter wallet on Testnet.
   They must connect their own wallet first and use the same passphrase-only
   options, not the first signer's wallet address.
6. Return the resulting XDR. Some Freighter versions return only the new signature;
   the operator may combine separately returned signatures only after verifying
   that both envelopes have the identical approved hash and both signatures verify.
   **Do not submit it yourself.**
   The verification helper will reject a modified transaction, test the actual
   one-signature envelope first, then submit the two-signature envelope and
   verify exact payout deltas. No private key or recovery phrase is needed.

Any two of Ayoub, Ace and King can sign. Repeating one person's signature does
not satisfy the quorum.

The frozen transaction reserves resource headroom for signature collection.
Its maximum fee is **10.001 free test XLM**, paid by the dedicated settlement
fixture account, not the individual signing wallets. This is not a production
fee policy. Lab's earlier re-simulation changed fees/resources, so all previous
envelopes and signatures are superseded; both members must sign this new hash.

Official instructions:
[Freighter's signing API guide](https://docs.freighter.app/extension-freighter-api/signing).

## Operator commands

Use the application workspace's installed Node runtime and Stellar SDK:

```bash
node --test contracts/togetherfi-stellar/scripts/testnet-common.test.mjs
node contracts/togetherfi-stellar/scripts/submit-testnet-completion.mjs /path/to/two-member-signed.xdr.txt
```

For a standalone source-repository checkout, run `npm ci --ignore-scripts` first
and use paths starting with `scripts/` instead of `contracts/togetherfi-stellar/scripts/`.
The SDK dependency is pinned in the Node manifest and lockfile. `npm test` runs
the eight offline approval-guard tests; it does not deploy or fund anything.

If signatures were not collected before the deadline:

```bash
node contracts/togetherfi-stellar/scripts/refresh-testnet-approval.mjs --refresh-testnet-approval
```

A refresh updates the manifest/file and invalidates the old envelope's collected
signatures. Publish the new file and hash before collecting signatures again.
The bootstrap deployment command must not be rerun over an existing manifest.