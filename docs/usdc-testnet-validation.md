# Circle USDC validation, Stellar Testnet only

The owner authorized a Circle USDC test on Stellar Testnet. No mainnet operation,
real funds, production wallet configuration or automatic replacement of historical
XLM fixtures is authorized.

## Asset identity

Source: [Circle USDC contract addresses](https://developers.circle.com/stablecoins/usdc-contract-addresses).
SAC addresses are derived from the exact code, issuer and network with the SDK,
not inferred from the ticker.

| Network | Code | Issuer | Stellar Asset Contract | Decimals |
|---|---|---|---|---:|
| Testnet | USDC | `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5` | `CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA` | 7 |
| Mainnet, metadata only | USDC | `GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN` | `CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75` | 7 |

## Bounded wallet smoke

```sh
node scripts/testnet-usdc-smoke.mjs --execute-testnet
```

The runner creates disposable testnet wallets and USDC trustlines, verifies the
RPC network and SAC metadata, and waits at most 30 minutes for human-completed
funding at [Circle's free faucet](https://faucet.circle.com/). Do not bypass its
CAPTCHA or request real USDC. After funding, the runner attempts exactly
0.0100000 testnet USDC via the actual SAC and checks exact sender/recipient
balance deltas. Public signed envelopes are saved before broadcast; fixture
seeds stay only in process memory. Evidence must not be overwritten on rerun.

A wallet smoke does not demonstrate campaign funding, activation, quorum
release, pool credits, refunds, or production readiness. A pending faucet wait
must never be described as a confirmed USDC payment.

## Fresh escrow acceptance, queued separately

1. Finish the current source reviews and use the exact reviewed WASM hashes.
2. Produce CI-attested builds and compare their hashes with deployment inputs.
3. Deploy separate USDC escrow and two pool instances; preserve all historical
   XLM instances, pending refund campaign and their original evidence.
4. Configure every instance with this testnet USDC SAC. Confirm bindings and
   independently controlled governance/settlement quorum.
5. Establish account trustlines and obtain free testnet USDC.
6. Confirm sponsor funding and activation, one-member rejection and a real
   two-member release with the designated members' own signatures.
7. Verify exact 82/10/5/2/1 USDC deltas, authenticated pool credits and liabilities.
8. Publish confirmed receipts and explorer source-verification status. Keep any
   outstanding real-expiry refund proof labeled pending.

Recipient delivery is on-chain USDC. No fiat provider is required or committed.
