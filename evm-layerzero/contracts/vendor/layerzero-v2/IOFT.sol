// SPDX-License-Identifier: MIT
// Source: LayerZero-Labs/LayerZero-v2 @ 9c741e7f9790639537b1710a203bcdfd73b0b9ac
pragma solidity >=0.8.0;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {
    MessagingFee,
    MessagingReceipt
} from "./ILayerZeroEndpointV2.sol";

struct SendParam {
    uint32 dstEid;
    bytes32 to;
    uint256 amountLD;
    uint256 minAmountLD;
    bytes extraOptions;
    bytes composeMsg;
    bytes oftCmd;
}

struct OFTLimit {
    uint256 minAmountLD;
    uint256 maxAmountLD;
}

struct OFTFeeDetail {
    int256 feeAmountLD;
    string description;
}

struct OFTReceipt {
    uint256 amountSentLD;
    uint256 amountReceivedLD;
}

/**
 * @notice Official LayerZero V2 OFT interface.
 * @dev This is the small, dependency-free copy of the pinned official
 *      interface used by the funding source.  Keeping it in the repository
 *      avoids importing the LayerZero package (which brings an ethers v5 peer
 *      dependency into this ethers v6 project).
 */
interface IOFT is IERC20 {
    event OFTSent(
        bytes32 indexed guid,
        uint32 indexed dstEid,
        address indexed from,
        bytes32 to,
        uint256 amountSentLD,
        uint256 amountReceivedLD
    );

    function oftVersion() external pure returns (bytes4 interfaceId, uint64 version);

    function token() external view returns (address);

    function approvalRequired() external view returns (bool);

    function sharedDecimals() external view returns (uint8);

    function quoteOFT(SendParam calldata _sendParam)
        external
        view
        returns (
            OFTLimit memory oftLimit,
            OFTFeeDetail[] memory oftFeeDetails,
            OFTReceipt memory oftReceipt
        );

    function quoteSend(SendParam calldata _sendParam, bool _payInLzToken)
        external
        view
        returns (MessagingFee memory msgFee);

    function send(
        SendParam calldata _sendParam,
        MessagingFee calldata _fee,
        address _refundAddress
    ) external payable returns (MessagingReceipt memory msgReceipt, OFTReceipt memory oftReceipt);
}