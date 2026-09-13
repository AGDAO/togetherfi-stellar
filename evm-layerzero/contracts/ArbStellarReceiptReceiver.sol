// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { Origin } from "./vendor/layerzero-v2/ILayerZeroEndpointV2.sol";
import { LayerZeroV2ReceiverBase } from "./vendor/layerzero-v2/LayerZeroV2ReceiverBase.sol";

/**
 * @title ArbStellarReceiptReceiver
 * @notice Financially inert LayerZero V2 receipt sink for final Stellar state.
 *
 * LayerZero delivery is informational only. This contract never sends ETH or
 * tokens, performs arbitrary calls, or changes any campaign settlement state.
 * Stellar remains authoritative for the campaign and operation represented by
 * each receipt.
 */
contract ArbStellarReceiptReceiver is LayerZeroV2ReceiverBase, Ownable {
    bytes4 public constant PAYLOAD_MAGIC = 0x54465a52; // "TFZR"
    uint8 public constant PAYLOAD_VERSION = 1;

    uint8 public constant STATUS_COMPLETED = 1;
    uint8 public constant STATUS_REFUNDED = 2;
    uint8 public constant STATUS_ANCHORED = 3;

    uint32 public immutable stellarEid;
    bytes32 public immutable stellarPeer;

    bool public paused;
    uint64 public lastNonce;

    mapping(uint8 => bool) public finalStatusAllowed;
    mapping(bytes32 => bool) public processedGuid;
    mapping(uint64 => mapping(uint32 => bool)) public operationProcessed;

    struct Receipt {
        uint64 campaignId;
        uint32 operationVersion;
        uint8 finalStatus;
        bytes32 stateHash;
        uint64 sourceNonce;
        uint32 sourceEid;
        bytes32 sourcePeer;
        uint64 receivedAt;
    }

    mapping(uint64 => mapping(uint32 => Receipt)) public receipts;

    error InvalidConfiguration();
    error Paused();
    error ValueNotAccepted();
    error InvalidGuid();
    error InvalidPayloadLength(uint256 length);
    error InvalidMagic(bytes4 magic);
    error InvalidVersion(uint8 version);
    error UnsupportedFinalStatus(uint8 status);
    error Replay(bytes32 guid);
    error InvalidOperationVersion();
    error DuplicateOperation(uint64 campaignId, uint32 operationVersion);
    error NonceOutOfOrder(uint64 expected, uint64 supplied);

    event PausedSet(bool paused);
    event ReceiptRecorded(
        bytes32 indexed guid,
        uint64 indexed campaignId,
        uint32 indexed operationVersion,
        uint8 finalStatus,
        uint64 sourceNonce
    );

    constructor(address _endpoint, uint32 _stellarEid, bytes32 _stellarPeer)
        LayerZeroV2ReceiverBase(_endpoint)
        Ownable(msg.sender)
    {
        if (_stellarPeer == bytes32(0)) revert InvalidConfiguration();
        stellarEid = _stellarEid;
        stellarPeer = _stellarPeer;
        peers[_stellarEid] = _stellarPeer;

        // Only terminal outcomes are accepted. There is intentionally no
        // owner setter: accepting an intermediate state would weaken the
        // receipt boundary.
        finalStatusAllowed[STATUS_COMPLETED] = true;
        finalStatusAllowed[STATUS_REFUNDED] = true;
        finalStatusAllowed[STATUS_ANCHORED] = true;
    }

    function setPaused(bool _paused) external onlyOwner {
        paused = _paused;
        emit PausedSet(_paused);
    }

    /**
     * @notice Returns the next source nonce required by this ordered path.
     */
    function nextNonce(uint32 _eid, bytes32 _sender)
        external
        view
        override
        returns (uint64)
    {
        if (_eid != stellarEid || _sender != stellarPeer) return 0;
        if (lastNonce == type(uint64).max) return type(uint64).max;
        return lastNonce + 1;
    }

    /// @notice Canonical encoder for the Stellar adapter's 50-byte TFZR payload.
    function encodeReceiptPayload(
        uint64 campaignId,
        uint32 operationVersion,
        uint8 finalStatus,
        bytes32 stateHash
    ) external pure returns (bytes memory) {
        if (
            finalStatus != STATUS_COMPLETED &&
            finalStatus != STATUS_REFUNDED &&
            finalStatus != STATUS_ANCHORED
        ) revert UnsupportedFinalStatus(finalStatus);
        if (operationVersion == 0) revert InvalidOperationVersion();
        return abi.encodePacked(PAYLOAD_MAGIC, PAYLOAD_VERSION, finalStatus, campaignId, operationVersion, stateHash);
    }

    function _lzReceive(
        Origin calldata _origin,
        bytes32 _guid,
        bytes calldata _message,
        address,
        bytes calldata
    ) internal override {
        if (paused) revert Paused();
        if (msg.value != 0) revert ValueNotAccepted();
        if (_guid == bytes32(0)) revert InvalidGuid();
        if (processedGuid[_guid]) revert Replay(_guid);
        if (_origin.srcEid != stellarEid || _origin.sender != stellarPeer) {
            revert OnlyPeer(_origin.srcEid, _origin.sender);
        }
        if (_origin.nonce == 0 || lastNonce == type(uint64).max || _origin.nonce != lastNonce + 1) {
            uint64 expected = lastNonce == type(uint64).max ? type(uint64).max : lastNonce + 1;
            revert NonceOutOfOrder(expected, _origin.nonce);
        }

        // TFZR is a fixed-width, byte-oriented protocol. Exact length means
        // malformed and trailing bytes cannot be accepted.
        if (_message.length != 50) revert InvalidPayloadLength(_message.length);
        bytes4 magic;
        uint8 version;
        uint8 finalStatus;
        uint64 campaignId;
        uint32 operationVersion;
        bytes32 stateHash;
        assembly {
            let ptr := _message.offset
            magic := calldataload(ptr)
            version := byte(4, calldataload(ptr))
            finalStatus := byte(5, calldataload(ptr))
            campaignId := shr(192, calldataload(add(ptr, 6)))
            operationVersion := shr(224, calldataload(add(ptr, 14)))
            stateHash := calldataload(add(ptr, 18))
        }
        if (magic != PAYLOAD_MAGIC) revert InvalidMagic(magic);
        if (version != PAYLOAD_VERSION) revert InvalidVersion(version);
        if (!finalStatusAllowed[finalStatus]) revert UnsupportedFinalStatus(finalStatus);
        if (operationVersion == 0) revert InvalidOperationVersion();
        if (operationProcessed[campaignId][operationVersion]) {
            revert DuplicateOperation(campaignId, operationVersion);
        }

        // Effects are applied only after every authentication and decoding
        // check has passed. There are no external calls in this function.
        processedGuid[_guid] = true;
        operationProcessed[campaignId][operationVersion] = true;
        lastNonce = _origin.nonce;
        receipts[campaignId][operationVersion] = Receipt({
            campaignId: campaignId,
            operationVersion: operationVersion,
            finalStatus: finalStatus,
            stateHash: stateHash,
            sourceNonce: _origin.nonce,
            sourceEid: _origin.srcEid,
            sourcePeer: _origin.sender,
            receivedAt: uint64(block.timestamp)
        });
        emit ReceiptRecorded(_guid, campaignId, operationVersion, finalStatus, _origin.nonce);
    }
}