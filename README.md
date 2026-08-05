# TogetherFi × Stellar — Soroban Contracts

[![Tests](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml/badge.svg)](https://github.com/AGDAO/togetherfi-stellar/actions/workflows/test.yml)

Two Soroban smart contracts deployed on Stellar testnet as part of the
[TogetherFi](https://togetherfi.io) Stellar Community Fund grant application.

---

## Deployed contracts (Stellar testnet)

| Contract | Address |
|---|---|
| Campaign Escrow | `CDEAOPTIFKCOFOOHNXXFLF3WAUSTXZKMIXN7NM5OM2BO3CLBX2AND34N` |
| Reputation Anchor | `CBJVDSS6VUWNORYG2W7ZMHPGMCBXQN5A2CKK5OSS7L2WFR6RSFLGQILC` |

---

## Campaign Escrow (`contracts/campaign_escrow`)

Holds USDC for a TogetherFi creator campaign and releases it atomically on
completion with the platform's fixed payout split:

| Recipient | Share |
|---|---|
| Creator wallet | 82.5% |
| Platform treasury | 10.0% |
| Revenue share pool | 5.0% + rounding dust |
| Referrer wallet | 2.5% (folds to pool if no referrer) |

### Functions

| Function | Caller | Action |
|---|---|---|
| `initialize(admin, usdc_token, platform_treasury, revenue_pool, default_referrer)` | admin | One-time setup |
| `fund(id, brand, amount)` | brand signs | Transfer USDC brand → contract |
| `complete(id, creator, referrer)` | admin only | Release with split atomically |
| `refund(id)` | admin only | Return full balance to brand |
| `get_campaign(id)` | anyone | Read campaign state |

### Verified testnet transactions

| Step | Transaction hash |
|---|---|
| Deploy | `54b6a37c3b95c86135aa8ae369f761f99a4204125bc025363bf2d3171101f553` |
| Initialize | `ff9e9c6347fd4acb86986dfd52e87a649b87b350338a096bcbb6e30c5b05b1bd` |
| Fund (end-to-end test) | `ebced8761f76e5e1bbdb5d0fe0859f095ce999e2fb5487358474399484d25c89` |
| Complete (end-to-end test) | `268ad813ee9192a9a6838b03173e3403d83df7bf1ab3cf17ae2c8790f574496b` |

Split verified on-chain: 82.5% creator / 10% platform treasury / 5% revenue pool / 2.5% referrer — escrow drained to zero.

> Note: XLM native asset used as USDC stand-in for the testnet test.
> Mainnet deployment will use the official Stellar USDC token contract on grant approval.

---

## Reputation Anchor (`contracts/reputation_anchor`)

Stores creator TogetherScores permanently on Stellar as the authoritative
on-chain identity record. Each entry includes an `arbitrum_tx_hash` field
as cross-chain proof, linking the Stellar anchor to the originating Arbitrum
Stylus transaction.

### Functions

| Function | Caller | Action |
|---|---|---|
| `initialize(admin)` | admin | One-time setup |
| `anchor_score(profile_id, creator, score, tier, arbitrum_tx_hash)` | admin only | Write/update score |
| `get_score(profile_id)` | anyone | Read one score record |
| `get_scores_batch(profile_ids)` | anyone | Batch read (missing entries skipped) |

### ScoreRecord fields

```rust
pub struct ScoreRecord {
    pub profile_id:       u64,    // TogetherFi DB profile ID
    pub creator:          Address, // Stellar G-key
    pub score:            u32,    // 0–1000 (TogetherScore v2)
    pub tier:             Symbol, // "bronze" | "silver" | "gold"
    pub arbitrum_tx_hash: String, // cross-chain proof — originating Arbitrum tx
    pub anchored_at:      u64,   // unix timestamp (ledger time)
}
```

### ScoreAnchored event

Emitted by `anchor_score` on every call. Indexed by `(profile_id, score, tier, anchored_at, arbitrum_tx_hash)`. Any Stellar explorer can verify the anchor on-chain.

### Verified testnet transactions

| Step | Transaction hash |
|---|---|
| WASM upload | `c5638580d9a661639d3dbcc8a23f86b51a769ee4da8f927246d07ece35281755` |
| Contract create | `05a0cc73f05f824615e92df8d4a67afe5f8866ec5c37a1841b59a034c00fa646` |
| Initialize | `d27a0fcf50cf4ca9bd1a3ac5bf64a686aef9b5006d3f71ae669378c4292b2483` |
| Test anchor (profile_id=1, score=750, tier=gold) | `c4fd49bef8164216ef5be5841c108c23b5a1fadb2e2b8d50de497d75b73e27e7` |

Verify: https://stellar.expert/explorer/testnet/tx/c4fd49bef8164216ef5be5841c108c23b5a1fadb2e2b8d50de497d75b73e27e7

---

## Building

### Toolchain requirement — CRITICAL

**Must compile with Rust 1.81. Rust 1.82+ produces WASM that soroban-sdk 21.x / stellar-cli 27 rejects at deploy-time:**

```
HostError: Error(WasmVm, InvalidAction)
Translation error: "reference-types not enabled: zero byte expected"
```

This is a rustc/LLVM codegen issue with `wasm32-unknown-unknown` targets in newer compilers. `-C target-feature=...` flags do not help; the fix is an older compiler.

### Build commands

```bash
# Campaign Escrow
cd contracts/campaign_escrow
cargo +1.81 build --target wasm32-unknown-unknown --release

# Reputation Anchor
cd contracts/reputation_anchor
cargo +1.81 build --target wasm32-unknown-unknown --release
```

### Running tests

```bash
# Campaign Escrow (4 tests)
cd contracts/campaign_escrow
cargo test

# Reputation Anchor (6 tests)
cd contracts/reputation_anchor
cargo test
```

### Deploying (stellar-cli 27)

```bash
# Upload and deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/<name>.wasm \
  --source <your-keypair-alias> \
  --network testnet
```

---

## Repository structure

```
contracts/
  campaign_escrow/
    Cargo.toml
    src/
      lib.rs    — contract implementation
      test.rs   — 4 integration tests
  reputation_anchor/
    Cargo.toml
    src/
      lib.rs    — contract implementation
      test.rs   — 6 integration tests
LICENSE
README.md
```

---

## License

MIT © 2024–2026 TogetherFi
