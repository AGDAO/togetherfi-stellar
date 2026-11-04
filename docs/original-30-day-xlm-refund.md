# Original 30-day XLM refund: confirmed

The original Campaign 2 refund was confirmed on **Stellar Testnet**, after its
unchanged 30-day expiry. This is not the separate accelerated 24-hour fixture.

| Evidence | Confirmed value |
| --- | --- |
| Escrow | `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ` |
| Campaign | `2` |
| Original funding | [e59cb68a…](https://stellar.expert/explorer/testnet/tx/e59cb68ac17dbd2845d2fd452a28d05cf0b7df6604849bc8d18b5872113a5e98), 4 October 2026 at 12:46:17 UTC |
| Refund eligibility | **3 November 2026 at 12:46:17 UTC** |
| Actual refund confirmation | **4 November 2026 at 20:22:30 UTC** |
| Refund transaction | [`34c53fb8d518c9a0f520be450c2178003c17a25a383bb779d0cb7f2a2923b5b6`](https://stellar.expert/explorer/testnet/tx/34c53fb8d518c9a0f520be450c2178003c17a25a383bb779d0cb7f2a2923b5b6) |
| Confirmed ledger | **5,560,022** |
| Original sponsor and refund recipient | `GBAFTNVGOI6JBJH5TKYDIQGKFFKP2YZIHAPKQMHNML5ELEMHLGCUIEQG` |
| Asset and refund | **1 free test XLM**, 10,000,000 base units, seven decimals |

## Verified reconciliation

| Item | Before | After | Change |
| --- | ---: | ---: | ---: |
| Sponsor balance, test XLM | 9,997.6457014 | 9,998.6457014 | +1 |
| Escrow token balance, test XLM | 1 | 0 | -1 |
| Escrow liability, test XLM | 1 | 0 | -1 |
| Campaign 2 status | Funded | Refunded | Terminal refund |

The RPC confirmed success and Horizon independently reported a successful
transaction. The deployed WASM matched the original recorded SHA-256
`1cfbad51dc1932e81de857b4a22402f883584ba4c6b89388aff53929c2e6b0eb`.
Campaigns 1, 3 and 4 and the escrow configuration were checked unchanged.
No expiry, signer, beneficiary or contract was changed.

A separate temporary caller funded by free Testnet Friendbot invoked the
permissionless `refund_expired(2)`. No existing signing key was used.
The caller paid the 8.2732331 test-XLM transaction fee; the sponsor received
the full 1 test XLM without that fee being deducted from the refund.
This Testnet fee is not a production cost estimate.

The [public manifest](../evidence/original-30-day-xlm-refund/manifest.json)
retains the signed transaction persisted before broadcast, confirmed receipt,
before/after balances, campaign states and reconciliation checks. No private
key is retained. The original deployment manifest remains historical evidence.

The separate original Circle-USDC campaign-3 expiry is **18 November 2026**.
This XLM receipt does not confirm that USDC refund, explorer source verification,
mainnet launch, or a grant payment.
