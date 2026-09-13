// SPDX-License-Identifier: MIT
// Source: LayerZero-Labs/LayerZero-v2 @ 9c741e7f9790639537b1710a203bcdfd73b0b9ac
pragma solidity >=0.8.0;

interface IMessagingContext {
    function isSendingMessage() external view returns (bool);
    function getSendContext() external view returns (uint32 dstEid, address sender);
}