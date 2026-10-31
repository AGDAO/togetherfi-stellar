# Historical deployed WASM: recovered source and build evidence

Evidence recovered on 31 October 2026. This note distinguishes recorded local
build evidence, deployed-byte identity, later CI attestation and explorer
source verification. None authorizes a deployment or a real-funds operation.

## Historical XLM deployment

The original build invocation was preserved. Earlier statements that it was
missing were incorrect: the deployment manifest alone did not contain it,
but the local build transcripts did.

Source commit:
[`5c507a3e49199ad772213ebb08667711e37a2746`](https://github.com/AGDAO/togetherfi-stellar/tree/5c507a3e49199ad772213ebb08667711e37a2746).

Original compiler and Cargo:

```text
rustc 1.81.0 (eeb90cda1 2024-09-04)
cargo 1.81.0 (2dbb1af80 2024-08-20)
target: wasm32-unknown-unknown
```

From each package directory, the 4 October transcript records:

```sh
cargo test --locked
cargo build --locked --release --target wasm32-unknown-unknown
```

The earlier 13 September reproducibility transcript records explicit toolchain
selection and the same final WASM hashes:

```sh
rustup run 1.81.0 cargo test --locked
rustup run 1.81.0 cargo build --locked --target wasm32-unknown-unknown --release
```

Use explicit toolchain selection when reproducing the recorded invocation.
The historical build was a raw Cargo release build, not a later Stellar CLI
metadata-injection and optimization pipeline.

| Package directory | Recorded WASM SHA-256 |
|---|---|
| `contracts/campaign_escrow` | `1cfbad51dc1932e81de857b4a22402f883584ba4c6b89388aff53929c2e6b0eb` |
| `contracts/contributor_pool` | `2f24a684373cfb2683912b1a85be0b8e4ac859fb64c15b14a52050579e99a865` |

These are the same hashes recorded by the RPC deployment verification in the
original manifest, for:

- Escrow: `CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ`
- NFT pool: `CAUPWNNQLQLURIJJTFW7LURJRD2OLGV4R5E6LNSUJXKSESRVGO54YP76`
- Revenue pool: `CD6TNSBSLAPXE4JXDBNTNEMMXH7DAWGUA3TR5HMP4QQIHCYM7544ED25`

On 31 October, the public GitHub contents API was used to retrieve the files
at that exact commit. Their SHA-256 values match the original transcript:

| File | SHA-256 |
|---|---|
| `contracts/campaign_escrow/Cargo.toml` | `bef87152433b806b1cd816367230fd4519845b5f947b7057ff228469c31d41db` |
| `contracts/campaign_escrow/Cargo.lock` | `abd7a4851f01e0c798eed6a7373ece704b61177d6dfa13eed55d02184373586b` |
| `contracts/campaign_escrow/src/lib.rs` | `55c5218a2f0b55116569ceb1398937ff20326cc4c68be7b47eacc6ddf4f51377` |
| `contracts/contributor_pool/Cargo.toml` | `7dcfcd44ac7ea03761e4ad581cfc6bbcf8e5de7f1211aa1b634126ea1532f934` |
| `contracts/contributor_pool/Cargo.lock` | `d1a00c6a94980e0b574b0d39a21454f82d86f453a245d4b8a3f3f1cde9b06633` |
| `contracts/contributor_pool/src/lib.rs` | `7b3d5e6a18ae45cd3849324411b374c69ca79b7d08bc09566c938af99421d72b` |

The original transcripts are retained alongside this note under
[`historical-build/campaign-escrow.txt`](historical-build/campaign-escrow.txt)
and [`historical-build/contributor-pool.txt`](historical-build/contributor-pool.txt).
They are preserved records, not a claim that a new build was performed today.

## Separate attested USDC generation

Source commit `a4d5d1eb80bb000c5abb6a11d714783546707618`, build run
[39325264354](https://github.com/AGDAO/togetherfi-stellar/actions/runs/39325264354),
and pinned builder
[`4a554f0e75c245cf4d3ea5f98f3a653d51edcd18`](https://github.com/stellar-expert/soroban-build-workflow/blob/4a554f0e75c245cf4d3ea5f98f3a653d51edcd18/.github/workflows/release.yml)
belong to the later, separate USDC fixtures.

Their pipeline runs `make ci-prepare`, `stellar contract build` with source,
home-domain and build-info metadata, and `stellar contract optimize`.
The escrow artifact is
`6f1a3e10abd79b76cbb4101f5ea42da5e51ad0d45950093e0cacf602fd51b0ff`;
the pool artifact is
`0842b07ff7056fcfce1df0e0586bf1779de57d4c6e4c89014a0af220d976c14f`.
They must not be substituted for the older XLM artifacts.

See [attested USDC evidence](attested-usdc-testnet-evidence.md) for these
deployments and the independently signed release.

## Explorer verification

The 19 October runner registration received HTTP 200 acknowledgements.
Acknowledgement is not an explorer source-match result. Workspace requests
receive HTTP 403, which does not establish whether a contract is verified.

The registration script now preserves the actual explorer responses for all
six deployed instances in `explorer-deployed-status.json` in the public
registration workflow artifact. Only an explicit successful explorer response
with `validation.status` equal to `verified` supports a verified-source claim.

Recovering the original invocation resolves the missing build-description
evidence. It does not retroactively attest the old build or manufacture an
explorer verification result. No deployed instance or artifact is replaced.
