// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {
    IOFT,
    SendParam,
    OFTLimit,
    OFTFeeDetail,
    OFTReceipt
} from "./vendor/layerzero-v2/IOFT.sol";
import {
    MessagingFee,
    MessagingReceipt
} from "./vendor/layerzero-v2/ILayerZeroEndpointV2.sol";

/**
 * @title ReviewedOFTAdapter
 * @notice Immutable, non-custodial boundary around one official LayerZero V2
 *         OFT.
 *
 * The official IOFT ABI intentionally does not include OApp endpoint/peer
 * getters. This local wrapper pins those route identities, the destination
 * EID, and the underlying OFT bytecode identity at construction. It has no
 * proxy or upgrade path. Tokens are pulled only for one send, approved for
 * exactly that send, forwarded to the underlying OFT, and any residual
 * balance causes the transaction to revert. `approvalRequired()` is the
 * wrapper's external capability and is always true; the separately pinned
 * `underlyingApprovalRequired` controls the underlying allowance.
 */
contract ReviewedOFTAdapter is IOFT {
    using SafeERC20 for IERC20;

    IOFT public immutable underlying;
    IERC20 public immutable underlyingToken;
    bytes32 public immutable underlyingCodehash;
    address public immutable routeEndpoint;
    uint32 public immutable routeDstEid;
    bytes32 public immutable routePeer;
    bytes4 public immutable routeOftInterfaceId;
    uint64 public immutable routeOftVersion;
    bool public immutable underlyingApprovalRequired;

    error InvalidRouteIdentity();
    error TokenBalanceInvariant(uint256 expected, uint256 actual);

    constructor(
        address underlying_,
        address endpoint_,
        uint32 dstEid_,
        bytes32 peer_,
        bytes32 expectedUnderlyingCodehash,
        bytes4 expectedOftInterfaceId,
        uint64 expectedOftVersion,
        bool expectedUnderlyingApprovalRequired
    ) {
        if (
            underlying_ == address(0) ||
            endpoint_ == address(0) ||
            dstEid_ == 0 ||
            peer_ == bytes32(0) ||
            expectedUnderlyingCodehash == bytes32(0) ||
            underlying_.code.length == 0 ||
            underlying_.codehash != expectedUnderlyingCodehash
        ) revert InvalidRouteIdentity();

        IOFT oft = IOFT(underlying_);
        address token_ = oft.token();
        (bytes4 interfaceId_, uint64 version_) = oft.oftVersion();
        bool approvalRequired_ = oft.approvalRequired();
        if (
            token_ == address(0) ||
            interfaceId_ != expectedOftInterfaceId ||
            version_ != expectedOftVersion ||
            approvalRequired_ != expectedUnderlyingApprovalRequired
        ) revert InvalidRouteIdentity();

        underlying = oft;
        underlyingToken = IERC20(token_);
        underlyingCodehash = expectedUnderlyingCodehash;
        routeEndpoint = endpoint_;
        routeDstEid = dstEid_;
        routePeer = peer_;
        routeOftInterfaceId = expectedOftInterfaceId;
        routeOftVersion = expectedOftVersion;
        underlyingApprovalRequired = approvalRequired_;
    }

    function token() external view override returns (address) {
        return address(underlyingToken);
    }

    function endpoint() external view returns (address) {
        return routeEndpoint;
    }

    function dstEid() external view returns (uint32) {
        return routeDstEid;
    }

    function peers(uint32 eid) external view returns (bytes32) {
        return eid == routeDstEid ? routePeer : bytes32(0);
    }

    function underlyingOFT() external view returns (address) {
        return address(underlying);
    }

    function underlyingOFTCodehash() external view returns (bytes32) {
        return underlyingCodehash;
    }

    function oftVersion() external pure override returns (bytes4 interfaceId, uint64 version) {
        // The official IOFT ABI declares this getter pure. The constructor
        // verifies these identities against the underlying before pinning.
        return (type(IOFT).interfaceId, 1);
    }

    function approvalRequired() external pure override returns (bool) {
        // ArbStellarSponsorFunding must approve this wrapper before send.
        return true;
    }

    function sharedDecimals() external view override returns (uint8) {
        return underlying.sharedDecimals();
    }

    function quoteOFT(SendParam calldata sendParam)
        external
        view
        override
        returns (
            OFTLimit memory limit,
            OFTFeeDetail[] memory details,
            OFTReceipt memory receipt
        )
    {
        return underlying.quoteOFT(sendParam);
    }

    function quoteSend(SendParam calldata sendParam, bool payInLzToken)
        external
        view
        override
        returns (MessagingFee memory fee)
    {
        return underlying.quoteSend(sendParam, payInLzToken);
    }

    function send(
        SendParam calldata sendParam,
        MessagingFee calldata fee,
        address refundAddress
    ) external payable override returns (MessagingReceipt memory receipt, OFTReceipt memory oftReceipt) {
        uint256 beforeBalance = underlyingToken.balanceOf(address(this));
        if (beforeBalance != 0) revert TokenBalanceInvariant(0, beforeBalance);
        underlyingToken.safeTransferFrom(msg.sender, address(this), sendParam.amountLD);
        if (underlyingApprovalRequired) {
            underlyingToken.forceApprove(address(underlying), sendParam.amountLD);
        }
        (receipt, oftReceipt) = underlying.send{ value: msg.value }(
            sendParam,
            fee,
            refundAddress
        );
        if (underlyingApprovalRequired) {
            underlyingToken.forceApprove(address(underlying), 0);
        }
        uint256 afterBalance = underlyingToken.balanceOf(address(this));
        if (afterBalance != 0) revert TokenBalanceInvariant(0, afterBalance);
    }

    function totalSupply() external view override returns (uint256) {
        return underlyingToken.totalSupply();
    }

    function balanceOf(address account) external view override returns (uint256) {
        return underlyingToken.balanceOf(account);
    }

    function transfer(address to, uint256 value) external override returns (bool) {
        return underlyingToken.transfer(to, value);
    }

    function allowance(address owner, address spender) external view override returns (uint256) {
        return underlyingToken.allowance(owner, spender);
    }

    function approve(address spender, uint256 value) external override returns (bool) {
        return underlyingToken.approve(spender, value);
    }

    function transferFrom(address from, address to, uint256 value) external override returns (bool) {
        return underlyingToken.transferFrom(from, to, value);
    }
}