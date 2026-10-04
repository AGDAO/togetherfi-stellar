# Sign the escrow v2 testnet completion

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
  `eabbc540c1f2e864679f16b089037ed076cf72c1c15e3fd5bc0092641c9a837a`.
- Initial signing deadline: **5 October 2026, 12:46:20 UTC**
  (**14:46:20 Berlin time**). If expired, request a refreshed envelope;
  all signers must sign the new version rather than combining different envelopes.

## Two signatures, collected in sequence

1. Open [the unsigned XDR file](../evidence/testnet-escrow-v2/complete-campaign-1.xdr.txt)
   and copy its full contents.
2. Open [Stellar Lab's transaction signer](https://lab.stellar.org/transaction/sign).
   Select **Testnet** and paste the XDR into the form.
3. Choose **Sign with Freighter**. Use your own testnet wallet and verify the
   details above before approving. Do not import the multisig source account or
   enter any secret keys into Lab.
4. Copy the **signed XDR returned by Lab** and send it to Ace or King.
5. That second signer pastes the **first signer's signed XDR**, not the original
   unsigned file, into Lab and signs with their own Freighter wallet on Testnet.
6. Return the final XDR containing both signatures. **Do not submit it from Lab.**
   The verification helper will reject a modified transaction, test the actual
   one-signature envelope first, then submit the two-signature envelope and
   verify exact payout deltas. No private key or recovery phrase is needed.

Any two of Ayoub, Ace and King can sign. Repeating one person's signature does
not satisfy the quorum.

Official instructions:
[Stellar's Freighter Soroban-XDR signing guide](https://developers.stellar.org/docs/build/guides/freighter/sign-soroban-xdrs).

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