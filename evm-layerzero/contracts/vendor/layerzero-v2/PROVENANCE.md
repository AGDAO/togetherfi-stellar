# LayerZero V2 EVM vendor provenance

The Solidity interface files in this directory are copied from the official
LayerZero V2 repository, pinned to commit
`9c741e7f9790639537b1710a203bcdfd73b0b9ac`:

<https://github.com/LayerZero-Labs/LayerZero-v2/tree/9c741e7f9790639537b1710a203bcdfd73b0b9ac/packages/layerzero-v2/evm>

This corresponds to the official LayerZero V2 EVM protocol/OApp source tree
used by the LayerZero packages at the time of this integration. The vendored
interfaces are MIT licensed by LayerZero Labs and retain their SPDX headers.
The receiver base is a receive-only adaptation of the official V2
`OAppReceiver` entry-point semantics: it intentionally omits OAppCore's
delegate and send/configuration calls because this contract has no send path
and must not make arbitrary external calls. Its `Origin`, `lzReceive`,
`allowInitializePath`, and `nextNonce` ABI types come from the pinned
`ILayerZeroReceiver.sol` and `ILayerZeroEndpointV2.sol`.

The pinned `@layerzerolabs/oft-evm@4.0.1` package is a dev dependency for
ABI compatibility tests. Its published OFT/OFTAdapter artifacts are
abstract/interface artifacts and do not contain deployable bytecode; the
package's `IOFT` ABI was compared with this vendored interface. Its peer
dependency tree includes an old ethers-v5 package, so installation uses the
project's existing npm lockfile with legacy peer resolution and the package
is not imported by production contracts.

`IOFT.sol` is the minimal official LayerZero V2 OFT interface from the same
pinned commit. It includes the canonical `SendParam`, `OFTReceipt`, quote, and
send ABI used by `ArbStellarSponsorFunding`. Since the official IOFT ABI does
not expose OApp endpoint/peer getters, production route binding uses the local
immutable `ReviewedOFTAdapter` boundary. The wrapper is non-upgradeable and
non-custodial; its constructor pins the official underlying OFT bytecode and
the operator-supplied endpoint/EID/peer route identities.