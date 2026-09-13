const { expect } = require("chai");
const { ethers } = require("hardhat");

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

describe("ArbStellarReceiptReceiver", function () {
  let owner;
  let attacker;
  let endpoint;
  let receiver;
  let peer;

  const eid = 10001;

  function origin(nonce, srcEid = eid, sender = peer) {
    return { srcEid, sender, nonce };
  }

  function payload(overrides = {}) {
    return ethers.solidityPacked(
      ["bytes4", "uint8", "uint8", "uint64", "uint32", "bytes32"],
      [
        overrides.magic ?? "0x54465a52",
        overrides.version ?? 1,
        overrides.status ?? 1,
        overrides.campaignId ?? 42,
        overrides.operationVersion ?? 3,
        overrides.stateHash ?? `0x${"07".repeat(32)}`,
      ]
    );
  }

  async function deliver(
    message = payload(),
    nonce = 1,
    guid = ethers.zeroPadValue(ethers.toBeHex(nonce), 32),
    messageOrigin = origin(nonce)
  ) {
    return endpoint.deliver(
      await receiver.getAddress(),
      messageOrigin,
      guid,
      message,
      ethers.ZeroAddress,
      "0x"
    );
  }

  beforeEach(async function () {
    [owner, attacker] = await ethers.getSigners();
    const Endpoint = await ethers.getContractFactory("MockLayerZeroEndpointV2");
    endpoint = await Endpoint.deploy();
    await endpoint.waitForDeployment();

    peer = ethers.zeroPadValue("0x1234", 32);
    const Receiver = await ethers.getContractFactory("ArbStellarReceiptReceiver");
    receiver = await Receiver.deploy(
      await endpoint.getAddress(),
      eid,
      peer
    );
    await receiver.waitForDeployment();
  });

  it("accepts only the configured endpoint and Stellar peer/EID", async function () {
    await deliver();
    expect(await receiver.lastNonce()).to.equal(1n);

    await expectRevert(
      receiver.connect(attacker).lzReceive(
        origin(2),
        ethers.zeroPadValue("0x03", 32),
        payload({ campaignId: 4 }),
        ethers.ZeroAddress,
        "0x"
      ),
      ethers.id("OnlyEndpoint(address)").slice(0, 10)
    );
    await expectRevert(
      deliver(payload(), 1, ethers.zeroPadValue("0x05", 32), origin(1, eid + 1)),
      "OnlyPeer"
    );
    await expectRevert(
      deliver(payload(), 1, ethers.zeroPadValue("0x06", 32), origin(1, eid, ethers.zeroPadValue("0x99", 32))),
      "OnlyPeer"
    );
  });

  it("matches the Stellar adapter's byte-for-byte TFZR golden vector", async function () {
    const golden =
      `0x54465a520101000000000000002a00000003${"07".repeat(32)}`;
    expect(payload()).to.equal(golden);
    expect(
      payload({
        campaignId: 99,
        operationVersion: 4,
        status: 2,
        stateHash: `0x${"02".repeat(32)}`,
      })
    ).to.equal(`0x54465a520102000000000000006300000004${"02".repeat(32)}`);
    expect(
      payload({
        campaignId: 1,
        operationVersion: 1,
        status: 3,
        stateHash: `0x${"ff".repeat(32)}`,
      })
    ).to.equal(`0x54465a520103000000000000000100000001${"ff".repeat(32)}`);
    await deliver(golden);
    const stored = await receiver.receipts(42, 3);
    expect(stored.campaignId).to.equal(42n);
    expect(stored.operationVersion).to.equal(3n);
    expect(stored.finalStatus).to.equal(1n);
    expect(stored.stateHash).to.equal(`0x${"07".repeat(32)}`);
  });

  it("rejects wrong TFZR magic/version, malformed payloads, and unsupported status", async function () {
    await expectRevert(deliver(payload({ version: 2 })), "InvalidVersion");
    await expectRevert(deliver(payload({ magic: "0x54464a52" /* replaced below */ }).replace("54465a52", "54465a53")), "InvalidMagic");
    await expectRevert(deliver(ethers.concat([payload(), "0x00"])), "InvalidPayloadLength");
    await expectRevert(deliver(payload({ status: 0 })), "UnsupportedFinalStatus");
    await expectRevert(deliver(payload({ operationVersion: 0 })), "InvalidOperationVersion");
  });

  it("rejects replayed GUIDs, duplicate operations, and out-of-order nonces", async function () {
    const first = payload();
    await deliver(first, 1, ethers.zeroPadValue("0xa1", 32));
    await expectRevert(deliver(first, 1, ethers.zeroPadValue("0xa1", 32)), "Replay");

    await expectRevert(
      deliver(payload(), 2, ethers.zeroPadValue("0xa2", 32)),
      "DuplicateOperation"
    );
    await expectRevert(
      deliver(payload({ campaignId: 3 }), 4, ethers.zeroPadValue("0xa4", 32)),
      "NonceOutOfOrder"
    );
  });

  it("can be paused by the owner and remains value-inert", async function () {
    await receiver.connect(owner).setPaused(true);
    await expectRevert(deliver(), "Paused");
    await receiver.connect(owner).setPaused(false);
    await deliver();
    expect(await ethers.provider.getBalance(await receiver.getAddress())).to.equal(0n);

    await expectRevert(
      owner.sendTransaction({ to: await receiver.getAddress(), value: 1n }),
      ""
    );
    await expectRevert(
      endpoint.deliver(
        await receiver.getAddress(),
        origin(2),
        ethers.zeroPadValue("0xb2", 32),
        payload({ campaignId: 178 }),
        ethers.ZeroAddress,
        "0x",
        { value: 1n }
      ),
      "ValueNotAccepted"
    );
    expect(await ethers.provider.getBalance(await receiver.getAddress())).to.equal(0n);
  });
});