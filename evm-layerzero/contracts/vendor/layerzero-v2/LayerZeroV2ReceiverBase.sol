// SPDX-License-Identifier: MIT
// Receiver entry-point types and semantics follow LayerZero V2 at:
// 9c741e7f9790639537b1710a203bcdfd73b0b9ac
pragma solidity ^0.8.23;

import { ILayerZeroEndpointV2, Origin } from "./ILayerZeroEndpointV2.sol";
import { ILayerZeroReceiver } from "./ILayerZeroReceiver.sol";

/**
 * @dev The receive-only subset of the LayerZero V2 OApp receiver.
 *
 * This deliberately does not include OAppCore's delegate/configuration calls:
 * this adapter has no send path and must not make arbitrary external calls.
 * The public entry point and Origin type are the official V2 definitions.
 */
abstract contract LayerZeroV2ReceiverBase is ILayerZeroReceiver {
    ILayerZeroEndpointV2 public immutable endpoint;
    mapping(uint32 => bytes32) public peers;

    error ZeroEndpoint();
    error OnlyEndpoint(address caller);
    error OnlyPeer(uint32 srcEid, bytes32 sender);

    constructor(address _endpoint) {
        if (_endpoint == address(0)) revert ZeroEndpoint();
        endpoint = ILayerZeroEndpointV2(_endpoint);
    }

    modifier onlyEndpoint() {
        if (msg.sender != address(endpoint)) revert OnlyEndpoint(msg.sender);
        _;
    }

    /**
     * @inheritdoc ILayerZeroReceiver
     * @dev This is payable because the official LayerZero V2 receiver ABI is
     * payable. Implementations may (and this adapter does) reject value.
     */
    function lzReceive(
        Origin calldata _origin,
        bytes32 _guid,
        bytes calldata _message,
        address _executor,
        bytes calldata _extraData
    ) external payable virtual override onlyEndpoint {
        if (peers[_origin.srcEid] == bytes32(0) || peers[_origin.srcEid] != _origin.sender) {
            revert OnlyPeer(_origin.srcEid, _origin.sender);
        }
        _lzReceive(_origin, _guid, _message, _executor, _extraData);
    }

    function allowInitializePath(Origin calldata _origin)
        external
        view
        virtual
        override
        returns (bool)
    {
        return peers[_origin.srcEid] != bytes32(0) && peers[_origin.srcEid] == _origin.sender;
    }

    /// @dev Zero means that a path has no ordered enforcement in LayerZero.
    function nextNonce(uint32, bytes32) external view virtual override returns (uint64) {
        return 0;
    }

    function _lzReceive(
        Origin calldata _origin,
        bytes32 _guid,
        bytes calldata _message,
        address _executor,
        bytes calldata _extraData
    ) internal virtual;
}