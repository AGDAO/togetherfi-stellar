const { expect } = require("chai");
const { ethers } = require("hardhat");
const officialIOFTArtifact = require("@layerzerolabs/oft-evm/artifacts/IOFT.sol/IOFT.json");

async function expectRevert(promise, fragment) {
  let threw = false;
  try {
    await promise;
  } catch (error) {
    threw = true;
    if (fragment && !error.message.includes(fragment)) {
      throw new Error(`Expected "${fragment}" but got:\n${error.message}`);
    }
  }
  if (!threw) throw new Error("Expected transaction to revert");
}

describe("ArbStellarSponsorFunding", function () {
  let owner;
  let sponsor;
  let other;
  let token;
  let oft;
  let adapter;
  let funding;

  const stellarEid = 10001;
  const fundingInbox = ethers.zeroPadValue("0xab", 32);
  const campaign = 17n;
  const operation = 2n;
  const sponsorAddress = `0x00${"11".repeat(32)}`;
  const sponsorReference = "0x123456";
  const assetDomain = ethers.keccak256(ethers.toUtf8Bytes("source-asset"));
  const escrowAssetDomain = ethers.keccak256(ethers.toUtf8Bytes("escrow-asset"));
  const amount = 1_000_000n;
  const minimum = 990_000n;
  const fee = 12345n;

  async function deadline() {
    const block = await ethers.provider.getBlock("latest");
    return BigInt(block.timestamp) + 3600n;
  }

  beforeEach(async function () {
    [owner, sponsor, other] = await ethers.getSigners();
    const Token = await ethers.getContractFactory("MockERC20");
    token = await Token.deploy("Sponsor Token", "SPON", 6);
    await token.waitForDeployment();
    const Oft = await ethers.getContractFactory("MockOFT");
    oft = await Oft.deploy(await token.getAddress());
    await oft.waitForDeployment();
    const underlyingAddress = await oft.getAddress();
    const underlyingCodehash = ethers.keccak256(await ethers.provider.getCode(underlyingAddress));
    const [underlyingInterfaceId, underlyingVersion] = await oft.oftVersion();
    const Adapter = await ethers.getContractFactory("ReviewedOFTAdapter");
    adapter = await Adapter.deploy(
      underlyingAddress,
      await oft.endpoint(),
      stellarEid,
      await oft.peers(stellarEid),
      underlyingCodehash,
      underlyingInterfaceId,
      underlyingVersion,
      true,
    );
    await adapter.waitForDeployment();
    const Funding = await ethers.getContractFactory("ArbStellarSponsorFunding");
    funding = await Funding.deploy(stellarEid, fundingInbox);
    await funding.waitForDeployment();
    await funding.setTokenAllowed(await token.getAddress(), true);
    const adapterAddress = await adapter.getAddress();
    const adapterCodehash = ethers.keccak256(await ethers.provider.getCode(adapterAddress));
    await funding.proposeAdapterRoute(
      adapterAddress,
      await token.getAddress(),
      await adapter.endpoint(),
      await adapter.dstEid(),
      await adapter.peers(stellarEid),
      underlyingInterfaceId,
      underlyingVersion,
      true,
      true,
      adapterCodehash,
      underlyingAddress,
      underlyingCodehash,
    );
    await ethers.provider.send("evm_increaseTime", [86401]);
    await ethers.provider.send("evm_mine");
    await funding.activateAdapterRoute(adapterAddress);
    await token.mint(sponsor.address, amount * 3n);
    await token.connect(sponsor).approve(await funding.getAddress(), amount * 3n);
  });

  async function create(campaignId = campaign, operationVersion = operation, extra = {}) {
    const intentDeadline = extra.deadline ?? await deadline();
    const authorizationNonce = extra.authorizationNonce ?? (campaignId * 1000n + operationVersion);
    const intentAdapter = extra.adapter ?? await adapter.getAddress();
    await funding.setCampaignAuthority(campaignId, owner.address);
    const network = await ethers.provider.getNetwork();
    const domain = {
      name: "ArbStellarSponsorFunding",
      version: "1",
      chainId: network.chainId,
      verifyingContract: await funding.getAddress(),
    };
    const types = {
      FundingIntentAuthorization: [
        { name: "version", type: "uint16" },
        { name: "campaignId", type: "uint64" },
        { name: "operationVersion", type: "uint32" },
        { name: "sponsor", type: "address" },
        { name: "token", type: "address" },
        { name: "amountLD", type: "uint256" },
        { name: "adapter", type: "address" },
        { name: "sponsorAddress", type: "bytes" },
        { name: "sponsorReference", type: "bytes" },
        { name: "minAmountLD", type: "uint256" },
        { name: "deadline", type: "uint256" },
        { name: "assetDomain", type: "bytes32" },
        { name: "escrowAssetDomain", type: "bytes32" },
        { name: "nonce", type: "uint256" },
      ],
    };
    const values = {
      version: 1,
      campaignId,
      operationVersion,
      sponsor: sponsor.address,
      token: await token.getAddress(),
      amountLD: extra.amount ?? amount,
      adapter: intentAdapter,
      sponsorAddress: extra.sponsorAddress ?? sponsorAddress,
      sponsorReference: ethers.getBytes(extra.sponsorReference ?? sponsorReference),
      minAmountLD: extra.minimum ?? minimum,
      deadline: intentDeadline,
      assetDomain: extra.assetDomain ?? assetDomain,
      escrowAssetDomain: extra.escrowAssetDomain ?? escrowAssetDomain,
      nonce: authorizationNonce,
    };
    const authorization = await owner.signTypedData(domain, types, values);
    return funding.connect(sponsor).createFundingIntent(
      campaignId,
      operationVersion,
      await token.getAddress(),
      extra.amount ?? amount,
      intentAdapter,
      extra.sponsorAddress ?? sponsorAddress,
      extra.sponsorReference ?? sponsorReference,
      extra.minimum ?? minimum,
      intentDeadline,
      extra.assetDomain ?? assetDomain,
      extra.escrowAssetDomain ?? escrowAssetDomain,
      authorizationNonce,
      authorization
    );
  }

  it("authenticates the sponsor and isolates one versioned intent", async function () {
    const createTx = await create();
    const createReceipt = await createTx.wait();
    const created = createReceipt.logs
      .map((log) => {
        try { return funding.interface.parseLog(log); } catch { return null; }
      })
      .find((parsed) => parsed?.name === "FundingIntentCreated");
    expect(created.args.authorizationNonce).to.equal(campaign * 1000n + operation);
    const intent = await funding.intents(campaign, operation);
    expect(intent.version).to.equal(1n);
    expect(intent.status).to.equal(1n);
    expect(intent.authorizationNonce).to.equal(campaign * 1000n + operation);
    expect(intent.sponsor).to.equal(sponsor.address);
    expect(await funding.localEscrow(await token.getAddress())).to.equal(amount);
    expect(await token.balanceOf(await funding.getAddress())).to.equal(amount);

    await expectRevert(create(), "DuplicateIntent");
    await expectRevert(
      funding.connect(other).createFundingIntent(
        campaign,
        operation,
        await token.getAddress(),
        amount,
        await adapter.getAddress(),
        sponsorAddress,
        sponsorReference,
        minimum,
        await deadline(),
         assetDomain,
         escrowAssetDomain,
         1n,
         "0x"
      ),
      "DuplicateIntent"
    );
    await expectRevert(
      funding.connect(other).refundFundingIntent(campaign, operation),
      "UnauthorizedSponsor"
    );
  });

  it("has no campaign authority configured by default", async function () {
    await expectRevert(
      funding.connect(sponsor).createFundingIntent(
        campaign,
        operation,
        await token.getAddress(),
        amount,
        await adapter.getAddress(),
        sponsorAddress,
        sponsorReference,
        minimum,
        await deadline(),
        assetDomain,
        escrowAssetDomain,
        0n,
        "0x"
      ),
      "CampaignAuthorityNotConfigured"
    );
  });

  it("matches the pinned official OFT ABI at the wrapper boundary", async function () {
    const official = new ethers.Interface(officialIOFTArtifact.abi);
    for (const name of ["token", "oftVersion", "approvalRequired", "sharedDecimals", "quoteOFT", "quoteSend", "send"]) {
      expect(adapter.interface.getFunction(name).selector).to.equal(
        official.getFunction(name).selector
      );
    }
    expect(await adapter.underlyingOFT()).to.equal(await oft.getAddress());
    expect(await adapter.underlyingOFTCodehash()).to.equal(
      ethers.keccak256(await ethers.provider.getCode(await oft.getAddress()))
    );
    expect(await adapter.approvalRequired()).to.equal(true);
    expect(await adapter.underlyingApprovalRequired()).to.equal(true);
  });

  it("creates and submits through a wrapper whose official OFT does not require approval", async function () {
    const Oft = await ethers.getContractFactory("MockOFT");
    const underlyingFalse = await Oft.deploy(await token.getAddress());
    await underlyingFalse.waitForDeployment();
    await underlyingFalse.setApprovalRequired(false);
    const underlyingAddress = await underlyingFalse.getAddress();
    const underlyingCodehash = ethers.keccak256(await ethers.provider.getCode(underlyingAddress));
    const [interfaceId, version] = await underlyingFalse.oftVersion();
    const Adapter = await ethers.getContractFactory("ReviewedOFTAdapter");
    const adapterFalse = await Adapter.deploy(
      underlyingAddress,
      await underlyingFalse.endpoint(),
      stellarEid,
      await underlyingFalse.peers(stellarEid),
      underlyingCodehash,
      interfaceId,
      version,
      false,
    );
    await adapterFalse.waitForDeployment();
    const adapterAddress = await adapterFalse.getAddress();
    const adapterCodehash = ethers.keccak256(await ethers.provider.getCode(adapterAddress));
    await funding.proposeAdapterRoute(
      adapterAddress,
      await token.getAddress(),
      await adapterFalse.endpoint(),
      await adapterFalse.dstEid(),
      await adapterFalse.peers(stellarEid),
      interfaceId,
      version,
      true,
      false,
      adapterCodehash,
      underlyingAddress,
      underlyingCodehash,
    );
    await ethers.provider.send("evm_increaseTime", [86401]);
    await ethers.provider.send("evm_mine");
    await funding.activateAdapterRoute(adapterAddress);
    expect(await adapterFalse.approvalRequired()).to.equal(true);
    expect(await adapterFalse.underlyingApprovalRequired()).to.equal(false);
    expect(await token.allowance(adapterAddress, underlyingAddress)).to.equal(0n);

    await create(40n, 1n, { adapter: adapterAddress });
    await funding.connect(sponsor).submitFundingIntent(40n, 1n, "0x");
    expect((await funding.intents(40n, 1n)).status).to.equal(2n);
    expect(await token.balanceOf(await funding.getAddress())).to.equal(0n);
    expect(await token.allowance(adapterAddress, underlyingAddress)).to.equal(0n);
  });

  it("quotes and submits with exact native fee and destination slippage floor", async function () {
    const intentDeadline = await deadline();
    await create(campaign, operation, { deadline: intentDeadline });
    await oft.setNativeFee(fee);
    const quoted = await funding.quoteFunding(campaign, operation, "0x1234");
    expect(quoted.nativeFee).to.equal(fee);

    await expectRevert(
      funding.connect(sponsor).submitFundingIntent(campaign, operation, "0x1234", {
        value: fee - 1n,
      }),
      "InvalidFee"
    );

    await funding.connect(sponsor).submitFundingIntent(campaign, operation, "0x1234", {
      value: fee,
    });
    const intent = await funding.intents(campaign, operation);
    expect(intent.status).to.equal(2n);
    expect(intent.messageGuid).to.not.equal(ethers.ZeroHash);
    expect(await funding.localEscrow(await token.getAddress())).to.equal(0n);
    expect(await token.balanceOf(await funding.getAddress())).to.equal(0n);
    expect(await token.balanceOf(await oft.getAddress())).to.equal(amount);
    expect(await oft.lastDstEid()).to.equal(BigInt(stellarEid));
    expect(await oft.lastRecipient()).to.equal(fundingInbox);
    expect(await oft.lastMinAmountLD()).to.equal(minimum);
    const expectedCompose = ethers.concat([
      "0x54465a46",
      "0x01",
      "0x01",
      ethers.toBeHex(campaign, 8),
      ethers.toBeHex(operation, 4),
      sponsorAddress,
      ethers.toBeHex(ethers.getBytes(sponsorReference).length, 4),
      sponsorReference,
      ethers.toBeHex(minimum, 16),
      ethers.toBeHex(intentDeadline, 8),
      assetDomain,
      escrowAssetDomain,
    ]);
    expect(await oft.lastComposeMsg()).to.equal(expectedCompose);
    expect((await oft.lastComposeMsg()).length).to.be.greaterThan(2);

    await expectRevert(
      funding.connect(sponsor).submitFundingIntent(campaign, operation, "0x", { value: fee }),
      "InvalidStatus"
    );
  });

  it("rejects a route that was explicitly deactivated", async function () {
    await create(30n, 1n);
    await funding.setAdapterAllowed(await adapter.getAddress(), false);
    await expectRevert(
      funding.connect(sponsor).submitFundingIntent(30n, 1n, "0x", { value: 0n }),
      "UnsupportedAdapter"
    );
  });

  it("requires independent route identities and a delayed activation", async function () {
    const adapterAddress = await adapter.getAddress();
    const adapterCodehash = ethers.keccak256(await ethers.provider.getCode(adapterAddress));
    const underlyingAddress = await oft.getAddress();
    const underlyingCodehash = ethers.keccak256(await ethers.provider.getCode(underlyingAddress));
    const [interfaceId, version] = await oft.oftVersion();
    await expectRevert(
      funding.proposeAdapterRoute(
        adapterAddress,
        await token.getAddress(),
        ethers.ZeroAddress,
        stellarEid,
        await adapter.peers(stellarEid),
        interfaceId,
        version,
        true,
        true,
        adapterCodehash,
        underlyingAddress,
        underlyingCodehash,
      ),
      "InvalidRouteIdentity"
    );
    await funding.proposeAdapterRoute(
      adapterAddress,
      await token.getAddress(),
      await adapter.endpoint(),
      await adapter.dstEid(),
      await adapter.peers(stellarEid),
      interfaceId,
      version,
      true,
      true,
      adapterCodehash,
      underlyingAddress,
      underlyingCodehash,
    );
    await expectRevert(
      funding.activateAdapterRoute(adapterAddress),
      "RouteActivationTooEarly"
    );
    await ethers.provider.send("evm_increaseTime", [86401]);
    await ethers.provider.send("evm_mine");
    await funding.activateAdapterRoute(adapterAddress);
    expect((await funding.adapterRoutes(adapterAddress)).allowed).to.equal(true);
  });

  it("rechecks the immutable wrapper and underlying identities on submission", async function () {
    await create(32n, 1n);
    await funding.connect(sponsor).submitFundingIntent(32n, 1n, "0x");
    expect((await funding.intents(32n, 1n)).status).to.equal(2n);
  });

  it("rejects an OFT receipt that does not meet the signed amount floor", async function () {
    await create(31n, 1n);
    await oft.setReceivedAmount(minimum - 1n);
    await expectRevert(
      funding.connect(sponsor).submitFundingIntent(31n, 1n, "0x", { value: 0n }),
      "InvalidOftReceipt"
    );
  });

  it("enforces allowlists, deadline, pause, and refund-before-submit only", async function () {
    const unknownToken = await (await ethers.getContractFactory("MockERC20"))
      .deploy("Unknown", "UNK", 6);
    await unknownToken.waitForDeployment();
    await expectRevert(
      funding.connect(sponsor).createFundingIntent(
        campaign,
        operation,
        await unknownToken.getAddress(),
        amount,
        await adapter.getAddress(),
        sponsorAddress,
        sponsorReference,
        minimum,
        await deadline(),
        assetDomain,
        escrowAssetDomain,
        1n,
        "0x"
      ),
      "UnsupportedToken"
    );

    await create();
    const before = await token.balanceOf(sponsor.address);
    await funding.connect(sponsor).refundFundingIntent(campaign, operation);
    expect(await token.balanceOf(sponsor.address)).to.equal(before + amount);
    expect(await funding.localEscrow(await token.getAddress())).to.equal(0n);
    expect((await funding.intents(campaign, operation)).status).to.equal(3n);
    await expectRevert(
      funding.connect(sponsor).refundFundingIntent(campaign, operation),
      "InvalidStatus"
    );

    await funding.setPaused(true);
    await expectRevert(
      create(68n, 69n),
      ""
    );
  });

  it("does not accept native donations and keeps the configured inbox destination", async function () {
    await expectRevert(
      sponsor.sendTransaction({ to: await funding.getAddress(), value: 1n }),
      "NativeValueNotAccepted"
    );

    await create(campaign, operation, {
      sponsorAddress: `0x01${"99".repeat(32)}`,
      sponsorReference,
    });
    const intent = await funding.intents(campaign, operation);
    expect(intent.sponsorAddress).to.equal(`0x01${"99".repeat(32)}`);
    expect(intent.sponsorReference).to.equal(sponsorReference);
  });

  it("validates the Soroban address, reference bound, and canonical domains", async function () {
    await expectRevert(
      create(18n, 1n, { sponsorAddress: `0x02${"11".repeat(32)}` }),
      "InvalidSponsorAddress"
    );
    await expectRevert(
      create(19n, 1n, { sponsorAddress: `0x00${"00".repeat(32)}` }),
      "InvalidSponsorAddress"
    );
    await expectRevert(
      create(20n, 1n, { sponsorReference: `0x${"aa".repeat(257)}` }),
      "InvalidSponsorReference"
    );
    await expectRevert(
      create(21n, 1n, { assetDomain: ethers.ZeroHash }),
      "InvalidAssetDomain"
    );
  });

  it("bounds source and minimum amounts to signed Stellar i128", async function () {
    const maxI128 = (1n << 127n) - 1n;
    await token.mint(sponsor.address, maxI128);
    await token.connect(sponsor).approve(await funding.getAddress(), maxI128);
    await create(22n, 1n, {
      amount: maxI128,
      minimum: maxI128,
    });
    const boundaryIntent = await funding.intents(22n, 1n);
    expect(boundaryIntent.amountLD).to.equal(maxI128);
    expect(boundaryIntent.minAmountLD).to.equal(maxI128);

    await expectRevert(
      create(23n, 1n, { amount: maxI128 + 1n, minimum: maxI128 }),
      "InvalidAmount"
    );
    await expectRevert(
      create(24n, 1n, { amount: maxI128, minimum: maxI128 + 1n }),
      "InvalidAmount"
    );
  });
});