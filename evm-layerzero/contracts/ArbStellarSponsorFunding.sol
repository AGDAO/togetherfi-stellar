// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { ReentrancyGuard } from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import { EIP712 } from "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import { ECDSA } from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {
    IOFT,
    SendParam,
    OFTReceipt
} from "./vendor/layerzero-v2/IOFT.sol";
import {
    MessagingFee,
    MessagingReceipt
} from "./vendor/layerzero-v2/ILayerZeroEndpointV2.sol";

/**
 * @title ArbStellarSponsorFunding
 * @notice Source-side, non-upgradeable funding intents for Stellar escrow.
 *
 * A sponsor first creates one intent for a campaign/operation version. The
 * exact ERC-20 amount is held against that intent, and only that sponsor can
 * submit it through an owner-approved OFT adapter. The compose instruction is
 * the canonical TFZF payload consumed by the Stellar FundingInbox. Stellar is
 * the authority for payout and refund outcomes: this contract has no payout,
 * receipt, or destination-refund command.
 *
 * The LayerZero message GUID and OFT nonce are recorded after submission.  An
 * EVM transaction hash is assigned by the source chain and is therefore
 * available on the FundingSubmitted event; sourceTxId is a deterministic
 * intent-transaction identity useful to indexers without pretending that
 * Solidity can read the eventual transaction hash.
 */
/**
 * @dev Mandatory, reviewed route surface supplied by the local immutable
 *      adapter wrapper. Official OFTs are not required to implement it.
 */
interface IReviewedOFTAdapter is IOFT {
    function endpoint() external view returns (address);
    function dstEid() external view returns (uint32);
    function peers(uint32 eid) external view returns (bytes32);
    function underlyingOFT() external view returns (address);
    function underlyingOFTCodehash() external view returns (bytes32);
    function underlyingApprovalRequired() external view returns (bool);
}

contract ArbStellarSponsorFunding is Ownable, ReentrancyGuard, EIP712 {
    using SafeERC20 for IERC20;

    uint16 public constant INTENT_VERSION = 1;
    bytes4 public constant COMPOSE_DOMAIN = 0x54465a46; // "TFZF"
    uint8 public constant COMPOSE_VERSION = 1;
    uint8 public constant COMPOSE_TYPE_FUNDING = 1;
    uint8 public constant STELLAR_ACCOUNT_KIND = 0;
    uint8 public constant STELLAR_CONTRACT_KIND = 1;
    uint256 public constant MAX_SPONSOR_REFERENCE = 256;
    // The official Stellar OFT compose envelope carries amount_ld as i128.
    // Keep both the sent and destination minimum amounts representable.
    uint256 public constant MAX_STELLAR_AMOUNT = uint256(uint128(type(int128).max));
    uint256 public constant ROUTE_TIMELOCK = 1 days;
    bytes32 public constant FUNDING_INTENT_AUTHORIZATION_TYPEHASH = keccak256(
        "FundingIntentAuthorization(uint16 version,uint64 campaignId,uint32 operationVersion,address sponsor,address token,uint256 amountLD,address adapter,bytes sponsorAddress,bytes sponsorReference,uint256 minAmountLD,uint256 deadline,bytes32 assetDomain,bytes32 escrowAssetDomain,uint256 nonce)"
    );
    uint8 public constant STATUS_FUNDED = 1;
    uint8 public constant STATUS_SUBMITTED = 2;
    uint8 public constant STATUS_REFUNDED = 3;
    uint32 public immutable stellarEid;
    bytes32 public immutable fundingInbox;

    struct FundingIntent {
        uint16 version;
        uint64 campaignId;
        uint32 operationVersion;
        address sponsor;
        address token;
        uint256 amountLD;
        address adapter;
        bytes sponsorAddress;
        bytes sponsorReference;
        uint256 minAmountLD;
        uint256 deadline;
        uint256 authorizationNonce;
        bytes32 assetDomain;
        bytes32 escrowAssetDomain;
        uint8 status;
        bytes32 sourceTxId;
        bytes32 messageGuid;
        uint64 messageNonce;
        uint256 sourceBlock;
        uint256 submittedAt;
    }

    mapping(uint64 => mapping(uint32 => FundingIntent)) public intents;
    mapping(address => bool) public tokenAllowed;
    mapping(address => bool) public adapterAllowed;
    mapping(address => uint256) public localEscrow;
    mapping(bytes32 => bool) public submittedGuid;
    mapping(uint64 => address) public campaignAuthority;
    mapping(uint64 => mapping(uint256 => bool)) public authorizationNonceUsed;

    struct AdapterRoute {
        bool allowed;
        address token;
        address endpoint;
        uint32 dstEid;
        bytes32 peer;
        bytes4 oftInterfaceId;
        uint64 oftVersion;
        bool approvalRequired;
        bool underlyingApprovalRequired;
        bytes32 adapterCodehash;
        address underlyingOFT;
        bytes32 underlyingOFTCodehash;
    }
    mapping(address => AdapterRoute) public adapterRoutes;

    struct AdapterRouteProposal {
        bool exists;
        address token;
        address endpoint;
        uint32 dstEid;
        bytes32 peer;
        bytes4 oftInterfaceId;
        uint64 oftVersion;
        bool approvalRequired;
        bool underlyingApprovalRequired;
        bytes32 adapterCodehash;
        address underlyingOFT;
        bytes32 underlyingOFTCodehash;
        uint256 activateAfter;
    }
    mapping(address => AdapterRouteProposal) public adapterRouteProposals;

    bool public paused;

    error Paused();
    error InvalidAddress();
    error InvalidCampaignOrOperation();
    error DuplicateIntent(uint64 campaignId, uint32 operationVersion);
    error UnsupportedToken(address token);
    error UnsupportedAdapter(address adapter);
    error AdapterTokenMismatch(address adapter, address expected, address actual);
    error InvalidAmount();
    error InvalidRecipient();
    error InvalidDeadline();
    error InvalidSponsorAddress();
    error InvalidSponsorReference();
    error InvalidAssetDomain();
    error UnauthorizedSponsor();
    error InvalidStatus(uint8 status);
    error DeadlineExpired(uint256 deadline);
    error InvalidFee(uint256 supplied, uint256 required);
    error LzTokenFeeUnsupported();
    error InvalidGuid();
    error TokenBalanceInvariant(address token, uint256 expected, uint256 actual);
    error NativeValueNotAccepted();
    error CampaignAuthorityNotConfigured(uint64 campaignId);
    error InvalidAuthorization();
    error AuthorizationNonceUsed(uint64 campaignId, uint256 nonce);
    error InvalidOftReceipt(uint256 amountSentLD, uint256 amountReceivedLD);
    error AdapterRouteMismatch(address adapter);
    error RouteProposalRequired(address adapter);
    error RouteActivationTooEarly(uint256 activateAfter);
    error InvalidRouteIdentity();

    event PausedSet(bool paused);
    event CampaignAuthoritySet(uint64 indexed campaignId, address indexed authority);
    event TokenAllowanceSet(address indexed token, bool allowed);
    event AdapterAllowanceSet(address indexed adapter, bool allowed);
    event AdapterRouteProposed(
        address indexed adapter,
        uint256 activateAfter,
        address token,
        address endpoint,
        uint32 dstEid,
        bytes32 peer,
        bytes4 oftInterfaceId,
        uint64 oftVersion,
        bool approvalRequired,
        bool underlyingApprovalRequired,
        bytes32 adapterCodehash,
        address underlyingOFT,
        bytes32 underlyingOFTCodehash
    );
    event AdapterRouteActivated(
        address indexed adapter,
        address token,
        address endpoint,
        uint32 dstEid,
        bytes32 peer,
        bytes4 oftInterfaceId,
        uint64 oftVersion,
        bool approvalRequired,
        bool underlyingApprovalRequired,
        bytes32 adapterCodehash,
        address underlyingOFT,
        bytes32 underlyingOFTCodehash
    );
    event FundingIntentCreated(
        uint64 indexed campaignId,
        uint32 indexed operationVersion,
        uint16 version,
        address indexed sponsor,
        address token,
        uint256 amountLD,
        address adapter,
        uint32 stellarEid,
        bytes32 fundingInbox,
        bytes sponsorAddress,
        bytes sponsorReference,
        uint256 minAmountLD,
        uint256 deadline,
        uint256 authorizationNonce,
        bytes32 assetDomain,
        bytes32 escrowAssetDomain,
        bytes32 sourceTxId
    );
    event FundingSubmitted(
        uint64 indexed campaignId,
        uint32 indexed operationVersion,
        bytes32 indexed messageGuid,
        uint64 messageNonce,
        uint256 amountSentLD,
        uint256 amountReceivedLD,
        uint256 nativeFee,
        bytes32 sourceTxId
    );
    event FundingRefunded(
        uint64 indexed campaignId,
        uint32 indexed operationVersion,
        address indexed sponsor,
        address token,
        uint256 amountLD
    );

    constructor(uint32 _stellarEid, bytes32 _fundingInbox)
        Ownable(msg.sender)
        EIP712("ArbStellarSponsorFunding", "1")
    {
        if (_stellarEid == 0 || _fundingInbox == bytes32(0)) revert InvalidAddress();
        stellarEid = _stellarEid;
        fundingInbox = _fundingInbox;
    }

    modifier whenNotPaused() {
        if (paused) revert Paused();
        _;
    }

    function setPaused(bool _paused) external onlyOwner {
        paused = _paused;
        emit PausedSet(_paused);
    }

    /**
     * @dev Allowlists are explicit owner controls and emit an auditable event.
     *      There is deliberately no arbitrary target or arbitrary calldata
     *      setter: only reviewed immutable adapters can be called by
     *      submitFundingIntent.
     */
    function setTokenAllowed(address token, bool allowed) external onlyOwner {
        if (token == address(0)) revert InvalidAddress();
        tokenAllowed[token] = allowed;
        emit TokenAllowanceSet(token, allowed);
    }

    /**
     * @dev Kept as a compatibility guard so an old operator cannot
     *      accidentally recreate the former immediate-allowlisting path.
     */
    function setAdapterAllowed(address adapter, bool allowed) external onlyOwner {
        if (adapter == address(0)) revert InvalidAddress();
        if (allowed) revert RouteProposalRequired(adapter);
        adapterAllowed[adapter] = false;
        adapterRoutes[adapter].allowed = false;
        emit AdapterAllowanceSet(adapter, false);
    }

    function proposeAdapterRoute(
        address adapter,
        address expectedToken,
        address expectedEndpoint,
        uint32 expectedDstEid,
        bytes32 expectedPeer,
        bytes4 expectedOftInterfaceId,
        uint64 expectedOftVersion,
        bool expectedApprovalRequired,
        bool expectedUnderlyingApprovalRequired,
        bytes32 expectedAdapterCodehash,
        address expectedUnderlyingOFT,
        bytes32 expectedUnderlyingOFTCodehash
    ) external onlyOwner {
        if (
            adapter == address(0) ||
            expectedToken == address(0) ||
            expectedEndpoint == address(0) ||
            expectedDstEid == 0 ||
            expectedPeer == bytes32(0) ||
            expectedOftVersion == 0 ||
            expectedAdapterCodehash == bytes32(0)
        ) revert InvalidRouteIdentity();
        if (expectedUnderlyingOFT == address(0) || expectedUnderlyingOFTCodehash == bytes32(0)) {
            revert InvalidRouteIdentity();
        }

        AdapterRoute memory expected = AdapterRoute({
            allowed: true,
            token: expectedToken,
            endpoint: expectedEndpoint,
            dstEid: expectedDstEid,
            peer: expectedPeer,
            oftInterfaceId: expectedOftInterfaceId,
            oftVersion: expectedOftVersion,
            approvalRequired: expectedApprovalRequired,
            underlyingApprovalRequired: expectedUnderlyingApprovalRequired,
            adapterCodehash: expectedAdapterCodehash,
            underlyingOFT: expectedUnderlyingOFT,
            underlyingOFTCodehash: expectedUnderlyingOFTCodehash
        });
        _validateRouteIdentity(adapter, expected);
        uint256 activateAfter = block.timestamp + ROUTE_TIMELOCK;
        adapterRouteProposals[adapter] = AdapterRouteProposal({
            exists: true,
            token: expectedToken,
            endpoint: expectedEndpoint,
            dstEid: expectedDstEid,
            peer: expectedPeer,
            oftInterfaceId: expectedOftInterfaceId,
            oftVersion: expectedOftVersion,
            approvalRequired: expectedApprovalRequired,
            underlyingApprovalRequired: expectedUnderlyingApprovalRequired,
            adapterCodehash: expectedAdapterCodehash,
            underlyingOFT: expectedUnderlyingOFT,
            underlyingOFTCodehash: expectedUnderlyingOFTCodehash,
            activateAfter: activateAfter
        });
        emit AdapterRouteProposed(
            adapter,
            activateAfter,
            expectedToken,
            expectedEndpoint,
            expectedDstEid,
            expectedPeer,
            expectedOftInterfaceId,
            expectedOftVersion,
            expectedApprovalRequired,
            expectedUnderlyingApprovalRequired,
            expectedAdapterCodehash,
            expectedUnderlyingOFT,
            expectedUnderlyingOFTCodehash
        );
    }

    function activateAdapterRoute(address adapter) external onlyOwner {
        AdapterRouteProposal memory proposal = adapterRouteProposals[adapter];
        if (!proposal.exists) revert RouteProposalRequired(adapter);
        if (block.timestamp < proposal.activateAfter) {
            revert RouteActivationTooEarly(proposal.activateAfter);
        }
        AdapterRoute memory route = AdapterRoute({
            allowed: true,
            token: proposal.token,
            endpoint: proposal.endpoint,
            dstEid: proposal.dstEid,
            peer: proposal.peer,
            oftInterfaceId: proposal.oftInterfaceId,
            oftVersion: proposal.oftVersion,
            approvalRequired: proposal.approvalRequired,
            underlyingApprovalRequired: proposal.underlyingApprovalRequired,
            adapterCodehash: proposal.adapterCodehash,
            underlyingOFT: proposal.underlyingOFT,
            underlyingOFTCodehash: proposal.underlyingOFTCodehash
        });
        _validateRouteIdentity(adapter, route);
        adapterRoutes[adapter] = route;
        adapterAllowed[adapter] = true;
        delete adapterRouteProposals[adapter];
        emit AdapterRouteActivated(
            adapter,
            route.token,
            route.endpoint,
            route.dstEid,
            route.peer,
            route.oftInterfaceId,
            route.oftVersion,
            route.approvalRequired,
            route.underlyingApprovalRequired,
            route.adapterCodehash,
            route.underlyingOFT,
            route.underlyingOFTCodehash
        );
        emit AdapterAllowanceSet(adapter, true);
    }

    function setCampaignAuthority(uint64 campaignId, address authority) external onlyOwner {
        if (campaignId == 0) revert InvalidCampaignOrOperation();
        campaignAuthority[campaignId] = authority;
        emit CampaignAuthoritySet(campaignId, authority);
    }

    /**
     * @notice Pull and escrow one sponsor-funded intent.
     * @dev The caller is the authenticated sponsor. The pair is permanently
     * keyed by campaignId and operationVersion; it can never be replaced.
     */
    function createFundingIntent(
        uint64 campaignId,
        uint32 operationVersion,
        address token,
        uint256 amountLD,
        address adapter,
        bytes calldata sponsorAddress,
        bytes calldata sponsorReference,
        uint256 minAmountLD,
        uint256 deadline,
        bytes32 assetDomain,
        bytes32 escrowAssetDomain,
        uint256 authorizationNonce,
        bytes calldata authorization
    ) external whenNotPaused nonReentrant {
        if (campaignId == 0 || operationVersion == 0) {
            revert InvalidCampaignOrOperation();
        }
        FundingIntent storage existing = intents[campaignId][operationVersion];
        if (existing.version != 0) revert DuplicateIntent(campaignId, operationVersion);
        if (!tokenAllowed[token]) revert UnsupportedToken(token);
        if (!adapterAllowed[adapter]) revert UnsupportedAdapter(adapter);
        if (token == address(0) || adapter == address(0)) revert InvalidAddress();
        if (
            amountLD == 0 ||
            amountLD > MAX_STELLAR_AMOUNT ||
            minAmountLD == 0 ||
            minAmountLD > amountLD ||
            minAmountLD > MAX_STELLAR_AMOUNT
        ) {
            revert InvalidAmount();
        }
        _validateSponsorAddress(sponsorAddress);
        if (sponsorReference.length > MAX_SPONSOR_REFERENCE) {
            revert InvalidSponsorReference();
        }
        if (assetDomain == bytes32(0) || escrowAssetDomain == bytes32(0)) {
            revert InvalidAssetDomain();
        }
        if (deadline <= block.timestamp || deadline > type(uint64).max) {
            revert InvalidDeadline();
        }

        address authority = campaignAuthority[campaignId];
        if (authority == address(0)) revert CampaignAuthorityNotConfigured(campaignId);
        if (authorizationNonceUsed[campaignId][authorizationNonce]) {
            revert AuthorizationNonceUsed(campaignId, authorizationNonce);
        }
        bytes32 authorizationDigest = _hashTypedDataV4(
            keccak256(
                abi.encode(
                    FUNDING_INTENT_AUTHORIZATION_TYPEHASH,
                    INTENT_VERSION,
                    campaignId,
                    operationVersion,
                    msg.sender,
                    token,
                    amountLD,
                    adapter,
                    keccak256(sponsorAddress),
                    keccak256(sponsorReference),
                    minAmountLD,
                    deadline,
                    assetDomain,
                    escrowAssetDomain,
                    authorizationNonce
                )
            )
        );
        (address recoveredAuthority, ECDSA.RecoverError recoverError,) =
            ECDSA.tryRecover(authorizationDigest, authorization);
        if (recoverError != ECDSA.RecoverError.NoError || recoveredAuthority != authority) {
            revert InvalidAuthorization();
        }
        authorizationNonceUsed[campaignId][authorizationNonce] = true;

        address adapterToken = IOFT(adapter).token();
        if (adapterToken != token) {
            revert AdapterTokenMismatch(adapter, token, adapterToken);
        }
        _validateActiveRoute(adapter, token);

        // A balance delta makes fee-on-transfer behavior explicit rather than
        // silently promising more on Stellar than the sponsor funded.
        IERC20 erc20 = IERC20(token);
        uint256 beforeBalance = erc20.balanceOf(address(this));
        erc20.safeTransferFrom(msg.sender, address(this), amountLD);
        uint256 afterBalance = erc20.balanceOf(address(this));
        if (
            afterBalance < beforeBalance ||
            afterBalance - beforeBalance != amountLD
        ) {
            revert TokenBalanceInvariant(token, beforeBalance + amountLD, afterBalance);
        }

        localEscrow[token] += amountLD;
        bytes32 sourceTxId = keccak256(
            abi.encode(
                block.chainid,
                address(this),
                block.number,
                msg.sender,
                campaignId,
                operationVersion,
                INTENT_VERSION
            )
        );
        existing.version = INTENT_VERSION;
        existing.campaignId = campaignId;
        existing.operationVersion = operationVersion;
        existing.sponsor = msg.sender;
        existing.token = token;
        existing.amountLD = amountLD;
        existing.adapter = adapter;
        existing.sponsorAddress = sponsorAddress;
        existing.sponsorReference = sponsorReference;
        existing.minAmountLD = minAmountLD;
        existing.deadline = deadline;
        existing.authorizationNonce = authorizationNonce;
        existing.assetDomain = assetDomain;
        existing.escrowAssetDomain = escrowAssetDomain;
        existing.status = STATUS_FUNDED;
        existing.sourceTxId = sourceTxId;
        existing.sourceBlock = block.number;

        emit FundingIntentCreated(
            campaignId,
            operationVersion,
            INTENT_VERSION,
            msg.sender,
            token,
            amountLD,
            adapter,
            stellarEid,
            fundingInbox,
            sponsorAddress,
            sponsorReference,
            minAmountLD,
            deadline,
            authorizationNonce,
            assetDomain,
            escrowAssetDomain,
            sourceTxId
        );
    }

    /**
     * @notice Return the exact native/LZ fee quote for a funded intent.
     * @dev Destination minAmountLD is the slippage protection.  It is part of
     *      the official SendParam and is never changed at submission time.
     */
    function quoteFunding(
        uint64 campaignId,
        uint32 operationVersion,
        bytes calldata extraOptions
    ) external view returns (MessagingFee memory fee) {
        FundingIntent storage intent = intents[campaignId][operationVersion];
        if (intent.version == 0) revert InvalidCampaignOrOperation();
        if (intent.status != STATUS_FUNDED) revert InvalidStatus(intent.status);
        if (block.timestamp > intent.deadline) revert DeadlineExpired(intent.deadline);
        SendParam memory sendParam = _sendParam(intent, extraOptions);
        return IOFT(intent.adapter).quoteSend(sendParam, false);
    }

    /**
     * @notice Submit one already-funded intent via its active reviewed adapter.
     * @param extraOptions Official LayerZero V2 execution options.
     *
     * composeMsg is the canonical TFZF funding instruction for the configured
     * Stellar FundingInbox. oftCmd remains empty; no arbitrary destination
     * command can be carried by this source contract.
     */
    function submitFundingIntent(
        uint64 campaignId,
        uint32 operationVersion,
        bytes calldata extraOptions
    ) external payable whenNotPaused nonReentrant {
        FundingIntent storage intent = intents[campaignId][operationVersion];
        if (intent.version == 0) revert InvalidCampaignOrOperation();
        if (intent.status != STATUS_FUNDED) revert InvalidStatus(intent.status);
        if (msg.sender != intent.sponsor) revert UnauthorizedSponsor();
        if (block.timestamp > intent.deadline) revert DeadlineExpired(intent.deadline);
        if (!tokenAllowed[intent.token]) revert UnsupportedToken(intent.token);
        _validateActiveRoute(intent.adapter, intent.token);

        SendParam memory sendParam = _sendParam(intent, extraOptions);
        MessagingFee memory fee = IOFT(intent.adapter).quoteSend(sendParam, false);
        if (fee.lzTokenFee != 0) revert LzTokenFeeUnsupported();
        if (msg.value != fee.nativeFee) revert InvalidFee(msg.value, fee.nativeFee);

        IERC20 token = IERC20(intent.token);
        uint256 beforeBalance = token.balanceOf(address(this));
        if (beforeBalance < intent.amountLD) {
            revert TokenBalanceInvariant(intent.token, localEscrow[intent.token], beforeBalance);
        }
        AdapterRoute memory route = adapterRoutes[intent.adapter];
        if (route.approvalRequired) token.forceApprove(intent.adapter, intent.amountLD);
        (MessagingReceipt memory receipt, OFTReceipt memory oftReceipt) =
            IOFT(intent.adapter).send{ value: fee.nativeFee }(
            sendParam,
            fee,
            payable(intent.sponsor)
        );
        if (route.approvalRequired) token.forceApprove(intent.adapter, 0);

        uint256 afterBalance = token.balanceOf(address(this));
        uint256 expectedBalance = beforeBalance - intent.amountLD;
        if (afterBalance != expectedBalance) {
            revert TokenBalanceInvariant(intent.token, expectedBalance, afterBalance);
        }
        if (receipt.guid == bytes32(0) || submittedGuid[receipt.guid]) {
            revert InvalidGuid();
        }
        if (
            oftReceipt.amountSentLD != intent.amountLD ||
            oftReceipt.amountReceivedLD < intent.minAmountLD ||
            oftReceipt.amountReceivedLD > intent.amountLD
        ) {
            revert InvalidOftReceipt(oftReceipt.amountSentLD, oftReceipt.amountReceivedLD);
        }

        localEscrow[intent.token] -= intent.amountLD;
        intent.status = STATUS_SUBMITTED;
        intent.messageGuid = receipt.guid;
        intent.messageNonce = receipt.nonce;
        intent.submittedAt = block.timestamp;
        submittedGuid[receipt.guid] = true;
        emit FundingSubmitted(
            campaignId,
            operationVersion,
            receipt.guid,
            receipt.nonce,
            intent.amountLD,
            oftReceipt.amountReceivedLD,
            fee.nativeFee,
            intent.sourceTxId
        );
    }

    /**
     * @notice Refund only a funded, not-yet-submitted intent.
     * @dev This does not model or command any Stellar payout/refund outcome.
     */
    function refundFundingIntent(uint64 campaignId, uint32 operationVersion)
        external
        nonReentrant
    {
        FundingIntent storage intent = intents[campaignId][operationVersion];
        if (intent.version == 0) revert InvalidCampaignOrOperation();
        if (intent.status != STATUS_FUNDED) revert InvalidStatus(intent.status);
        if (msg.sender != intent.sponsor) revert UnauthorizedSponsor();
        IERC20 token = IERC20(intent.token);
        uint256 balance = token.balanceOf(address(this));
        if (balance < intent.amountLD || localEscrow[intent.token] < intent.amountLD) {
            revert TokenBalanceInvariant(
                intent.token,
                localEscrow[intent.token],
                balance
            );
        }
        intent.status = STATUS_REFUNDED; // terminal local state before interaction
        localEscrow[intent.token] -= intent.amountLD;
        token.safeTransfer(intent.sponsor, intent.amountLD);
        emit FundingRefunded(
            campaignId,
            operationVersion,
            intent.sponsor,
            intent.token,
            intent.amountLD
        );
    }

    /**
     * @notice Canonically encode the TFZF funding compose instruction.
     * @dev Integer fields are packed big-endian by Solidity's abi.encodePacked.
     *      Source nonce, source EID, amount, and compose-from are deliberately
     *      absent: the official OFT/Stellar wrapper supplies those values.
     */
    function encodeComposeMsg(
        uint64 campaignId,
        uint32 operationVersion,
        bytes calldata sponsorAddress,
        bytes calldata sponsorReference,
        uint256 minAmountLD,
        uint256 deadline,
        bytes32 assetDomain,
        bytes32 escrowAssetDomain
    ) external pure returns (bytes memory) {
        if (operationVersion == 0) revert InvalidCampaignOrOperation();
        _validateSponsorAddress(sponsorAddress);
        if (sponsorReference.length > MAX_SPONSOR_REFERENCE) {
            revert InvalidSponsorReference();
        }
        if (
            minAmountLD == 0 ||
            minAmountLD > MAX_STELLAR_AMOUNT
        ) {
            revert InvalidAmount();
        }
        if (deadline == 0 || deadline > type(uint64).max) revert InvalidDeadline();
        if (assetDomain == bytes32(0) || escrowAssetDomain == bytes32(0)) {
            revert InvalidAssetDomain();
        }
        return _buildComposeMsg(
            campaignId,
            operationVersion,
            sponsorAddress,
            sponsorReference,
            minAmountLD,
            deadline,
            assetDomain,
            escrowAssetDomain
        );
    }

    function _sendParam(FundingIntent storage intent, bytes calldata extraOptions)
        internal
        view
        returns (SendParam memory)
    {
        bytes memory composeMsg = _buildComposeMsg(
            intent.campaignId,
            intent.operationVersion,
            intent.sponsorAddress,
            intent.sponsorReference,
            intent.minAmountLD,
            intent.deadline,
            intent.assetDomain,
            intent.escrowAssetDomain
        );
        return SendParam({
            dstEid: stellarEid,
            to: fundingInbox,
            amountLD: intent.amountLD,
            minAmountLD: intent.minAmountLD,
            extraOptions: extraOptions,
            composeMsg: composeMsg,
            oftCmd: bytes("")
        });
    }

    function _buildComposeMsg(
        uint64 campaignId,
        uint32 operationVersion,
        bytes memory sponsorAddress,
        bytes memory sponsorReference,
        uint256 minAmountLD,
        uint256 deadline,
        bytes32 assetDomain,
        bytes32 escrowAssetDomain
    ) internal pure returns (bytes memory) {
        return abi.encodePacked(
            COMPOSE_DOMAIN,
            COMPOSE_VERSION,
            COMPOSE_TYPE_FUNDING,
            campaignId,
            operationVersion,
            sponsorAddress,
            uint32(sponsorReference.length),
            sponsorReference,
            int128(int256(minAmountLD)),
            uint64(deadline),
            assetDomain,
            escrowAssetDomain
        );
    }

    function _validateSponsorAddress(bytes calldata sponsorAddress) internal pure {
        if (sponsorAddress.length != 33) revert InvalidSponsorAddress();
        uint8 kind = uint8(sponsorAddress[0]);
        if (kind != STELLAR_ACCOUNT_KIND && kind != STELLAR_CONTRACT_KIND) {
            revert InvalidSponsorAddress();
        }
        bool nonzero;
        for (uint256 i = 1; i < sponsorAddress.length; ++i) {
            if (sponsorAddress[i] != bytes1(0)) {
                nonzero = true;
                break;
            }
        }
        if (!nonzero) revert InvalidSponsorAddress();
    }

    struct RouteIdentity {
        address token;
        address endpoint;
        uint32 dstEid;
        bytes32 peer;
        bytes4 oftInterfaceId;
        uint64 oftVersion;
        bool approvalRequired;
        bool underlyingApprovalRequired;
        bytes32 adapterCodehash;
        address underlyingOFT;
        bytes32 underlyingOFTCodehash;
    }

    function _readRouteIdentity(address adapter)
        internal
        view
        returns (RouteIdentity memory identity)
    {
        if (adapter.code.length == 0) revert InvalidRouteIdentity();
        IReviewedOFTAdapter reviewed = IReviewedOFTAdapter(adapter);
        identity.token = reviewed.token();
        identity.endpoint = reviewed.endpoint();
        identity.dstEid = reviewed.dstEid();
        identity.peer = reviewed.peers(identity.dstEid);
        (identity.oftInterfaceId, identity.oftVersion) = reviewed.oftVersion();
        identity.approvalRequired = reviewed.approvalRequired();
        identity.underlyingApprovalRequired = reviewed.underlyingApprovalRequired();
        identity.adapterCodehash = adapter.codehash;
        identity.underlyingOFT = reviewed.underlyingOFT();
        identity.underlyingOFTCodehash = reviewed.underlyingOFTCodehash();
        if (
            identity.token == address(0) ||
            identity.endpoint == address(0) ||
            identity.dstEid != stellarEid ||
            identity.peer == bytes32(0) ||
            identity.oftVersion == 0 ||
            identity.adapterCodehash == bytes32(0) ||
            identity.underlyingOFT == address(0) ||
            identity.underlyingOFTCodehash == bytes32(0) ||
            identity.underlyingOFT.code.length == 0 ||
            identity.underlyingOFT.codehash != identity.underlyingOFTCodehash
        ) revert InvalidRouteIdentity();
    }

    function _validateRouteIdentity(address adapter, AdapterRoute memory expected)
        internal
        view
    {
        RouteIdentity memory live = _readRouteIdentity(adapter);
        if (
            live.token != expected.token ||
            live.endpoint != expected.endpoint ||
            live.dstEid != expected.dstEid ||
            live.peer != expected.peer ||
            live.oftInterfaceId != expected.oftInterfaceId ||
            live.oftVersion != expected.oftVersion ||
            live.approvalRequired != expected.approvalRequired ||
            live.underlyingApprovalRequired != expected.underlyingApprovalRequired ||
            live.adapterCodehash != expected.adapterCodehash ||
            live.underlyingOFT != expected.underlyingOFT ||
            live.underlyingOFTCodehash != expected.underlyingOFTCodehash
        ) revert AdapterRouteMismatch(adapter);
    }

    function _validateActiveRoute(address adapter, address token) internal view {
        if (!adapterAllowed[adapter]) revert UnsupportedAdapter(adapter);
        AdapterRoute memory route = adapterRoutes[adapter];
        if (!route.allowed || route.token != token) revert AdapterRouteMismatch(adapter);
        _validateRouteIdentity(adapter, route);
    }

    receive() external payable {
        revert NativeValueNotAccepted();
    }

    fallback() external payable {
        revert NativeValueNotAccepted();
    }
}