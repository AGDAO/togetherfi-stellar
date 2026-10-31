# Historical deployed WASM: recovered source and build evidence

Evidence recovered on 31 October 2026. This note distinguishes recorded local
build evidence, deployed-byte identity, later CI attestation and explorer
source verification. None authorizes a deployment or a real-funds operation.

## Requested reviewer evidence

The review request was: "StellarExpert: the contracts still show as
unverified. Send us the exact commit and build command used for the deployed
WASM."

This note supplies that requested commit and build evidence. The review
wording observes the unverified explorer status; it does not require changing
that badge before supplying the evidence. Explorer status remains explicitly
unverified and must not be described as verified.

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

A fresh read-only RPC check on 31 October retrieved the actual WASM bytes for
all three historical instances and all three later USDC instances. All six
hashes match their respective recorded artifacts. The
[machine-readable results](historical-build/live-rpc-check.json) include the
timestamp, instance IDs, expected and retrieved hashes, and byte lengths.
This is a fresh deployed-byte check, not a fresh compilation.

The check can be repeated from an application checkout with the Stellar SDK:

```sh
node scripts/check-deployed-wasm.mjs /path/to/new-live-rpc-check.json
```

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

The exact command arguments in the pinned builder for the Escrow package are:

```sh
# Working directory: contracts/campaign_escrow
# Stellar CLI: 22.0.1; Rust: 1.81.0; target: wasm32-unknown-unknown
make ci-prepare
BUILD_INFO='{"package":"campaign-escrow","make_target":"ci-prepare","relative_path":"contracts/campaign_escrow"}'
stellar contract build --package campaign-escrow --out-dir "$OUTPUT" \
  --meta source_repo=AGDAO/togetherfi-stellar \
  --meta home_domain=togetherfi.io \
  --meta build_info="$BUILD_INFO"
# The builder copies the output to its versioned release filename, then runs:
stellar contract optimize --wasm "$WASM_FILE_NAME" --wasm-out "$WASM_FILE_NAME"
```

`OUTPUT` and `WASM_FILE_NAME` are the original workflow's generated output
directory and versioned filename. The pinned workflow linked above records
their construction and the exact copy operation. For the Contributor Pool
job, the directory is `contracts/contributor_pool`, the package is
`contributor-pool`, and `BUILD_INFO` is
`{"package":"contributor-pool","make_target":"ci-prepare","relative_path":"contracts/contributor_pool"}`.
Both source commits, build generations and final hashes remain distinct.

## Explorer verification

The [31 October registration run](https://github.com/AGDAO/togetherfi-stellar/actions/runs/41212580007)
completed successfully. It cryptographically reverified all three newer
attested artifacts and received HTTP 200 `{"ok":1}` acknowledgements for their
registrations. It did not register the historical raw-Cargo artifacts as if
they were those later attested builds.

The same runner then received HTTP 200 for all six deployed-contract status
requests. Every response explicitly reported `validation.status: "unverified"`.
The API returned no further explanation for the unverified state.

Preserved evidence:

- [Registration requests and acknowledgements](historical-build/explorer-registration.json).
- [All six actual explorer responses](historical-build/explorer-deployed-status.json).
- [GitHub Actions run and downloadable verification artifacts](https://github.com/AGDAO/togetherfi-stellar/actions/runs/41212580007).

The initial retry failed before reaching StellarExpert because the downloader
ignored the runner's standard `GH_TOKEN`. The downloader now accepts that
standard variable as well as the existing local credential variable. This
fix does not weaken artifact attestation checks or modify workflow permissions.
The successful retry establishes registration acceptance and readable explorer
status, not source-match completion.

Only an explicit successful explorer response with `validation.status` equal
to `verified` supports a verified-source claim. No verified-source link is
available from the observed results; the explorer's source-match process
remains unverified. It must not be reported as complete merely because the
workflow passed, nor added as a new condition to the quoted commit/build request.

A [network-specific follow-on check](https://github.com/AGDAO/togetherfi-stellar/actions/runs/41213314702)
also submitted the three attested registrations to the testnet endpoint, not
only the public/mainnet endpoint used in the upstream example. All three
testnet registrations returned HTTP 200 `{"ok":1}`. Subsequent testnet contract
requests with a no-cache header still returned `unverified` for all six
instances. The run preserves `explorer-testnet-registration.json` alongside
the actual status responses. Changing the registration network did not change
the observed verification result.

The requested commit/build evidence is supplied above; the separate explorer
badge is not represented as a missing deliverable in the quoted review item.

Recovering the original invocation resolves the missing build-description
evidence. It does not retroactively attest the old build or manufacture an
explorer verification result. No deployed instance or artifact is replaced.
