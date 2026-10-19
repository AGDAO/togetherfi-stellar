// Fresh testnet deployment and automated fixtures; never old deployments or real funds.
import assert from 'node:assert/strict';
import { randomBytes } from 'node:crypto';
import { writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  Address, Operation, TransactionBuilder, nativeToScVal, scValToNative, xdr,
} from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, horizon, basicTransaction, addressValue, assertQuorum,
  contractId, submitPrepared, sleep,
} from './testnet-common.mjs';
import { deploymentProof, verifyDeploymentBindings } from './deployment-binding.mjs';
import { createUsdcFixture, sha256, feeCap, expectedUsdcSplit } from './usdc-fixture-runtime.mjs';

const args = process.argv.slice(2);
if (args.length !== 3 || args[0] !== '--execute-testnet' || args[1] !== '--attested-build') {
  throw new Error('Usage: --execute-testnet --attested-build <verified-build.json>');
}
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const out = resolve(ROOT, 'evidence/testnet-usdc-escrow',
  new Date().toISOString().replace(/[:.]/g, '-'));
const f = await createUsdcFixture(resolve(args[2]), resolve(out, 'manifest.json'));
const { manifest: m, accounts: a, keys, members } = f;
const u64 = n => nativeToScVal(n, { type: 'u64' });
const u32 = n => nativeToScVal(n, { type: 'u32' });
const i128 = n => nativeToScVal(n, { type: 'i128' });

async function configureAutomation() {
  const ops = [];
  for (const role of ['governance', 'settlement']) {
    for (const key of members) ops.push(Operation.setOptions({
      source: a[role], signer: { ed25519PublicKey: key.publicKey(), weight: 1 },
    }));
    ops.push(Operation.setOptions({
      source: a[role], masterWeight: 0, lowThreshold: 2, medThreshold: 2, highThreshold: 2,
    }));
  }
  const tx = await basicTransaction(a.deployer, ops);
  tx.sign(keys.deployer, keys.governance, keys.settlement);
  await submitPrepared(tx, 'dedicated_roles_to_disposable_2_of_3', m, f.path);
  for (const role of ['governance', 'settlement']) {
    const account = await horizon.loadAccount(a[role]);
    assertQuorum(account, members.map(k => k.publicKey()));
  }
  f.setAutomatedQuorum();
}
async function fund(id, amount) {
  await f.call(a.sponsor, m.contracts.escrow.id, 'fund',
    [u64(id), addressValue(a.sponsor), i128(amount)], `fund_usdc_campaign_${id}`);
}
async function activate(id) {
  await f.call(a.sponsor, m.contracts.escrow.id, 'activate', [
    u64(id), addressValue(a.creator), addressValue(a.referrer),
    nativeToScVal(Buffer.from('fresh-attested-usdc-testnet-lifecycle')),
  ], `activate_usdc_campaign_${id}`);
}
function assertSplit(before, after, total) {
  const expected = expectedUsdcSplit(total);
  for (const [name, value] of Object.entries(expected)) {
    assert.equal(BigInt(after[name]) - BigInt(before[name]), value, name + ' USDC delta');
  }
  assert.equal(BigInt(before.escrow) - BigInt(after.escrow), total);
  return Object.fromEntries(Object.entries(expected).map(([k, v]) => [k, String(v)]));
}
async function allocateAndClaim(name, amount) {
  const pool = m.contracts[name].id;
  assert.equal(BigInt(await f.view(pool, 'available_balance')), amount);
  await f.call(a.governance, pool, 'publish_manifest', [
    u64(1), nativeToScVal(randomBytes(32)), nativeToScVal(randomBytes(32)),
    u64(BigInt(Math.floor(Date.now() / 1000) + 86400)),
    xdr.ScVal.scvVec([addressValue(a.creator)]),
    xdr.ScVal.scvVec([i128(amount)]),
  ], `${name}_dual_authority_allocation`);
  const before = await f.usdcBalance(a.creator);
  await f.call(a.creator, pool, 'claim', [u64(1), u32(0)], `${name}_recipient_claim`);
  assert.equal(await f.usdcBalance(a.creator) - before, amount);
  assert.equal(BigInt(await f.view(pool, 'available_balance')), 0n);
  await f.reject(a.creator, pool, 'claim', [u64(1), u32(0)], 7, `${name}_completed_manifest_replay_rejected`);
}

try {
  await f.activateAccountsAndTrustlines();
  const hashes = {};
  for (const [name, wasm] of Object.entries(f.binaries)) {
    // Observed upload quote is ~13.75 free test XLM; only uploads get a 20-XLM cap.
    const uploaded = await f.send(a.deployer, Operation.uploadContractWasm({ wasm }),
      'upload_attested_' + name, 200_000_000n);
    hashes[name] = sha256(wasm);
    assert.equal(Buffer.from(scValToNative(uploaded.returnValue)).toString('hex'), hashes[name]);
  }
  for (const name of ['escrow', 'nft_pool', 'revenue_pool']) {
    const packageName = name === 'escrow' ? 'campaign_escrow' : 'contributor_pool';
    const salt = randomBytes(32);
    m.contracts[name] = { id: contractId(a.deployer, salt), salt: salt.toString('hex') };
    await f.save();
    const result = await f.send(a.deployer, Operation.createCustomContract({
      address: new Address(a.deployer), wasmHash: Buffer.from(hashes[packageName], 'hex'), salt,
    }), 'deploy_usdc_' + name);
    assert.equal(scValToNative(result.returnValue), m.contracts[name].id);
    const onchain = await server.getContractWasmByContractId(m.contracts[name].id);
    assert.equal(sha256(onchain), hashes[packageName], 'RPC bytes differ from attested release');
    m.contracts[name].rpc_wasm_sha256 = sha256(onchain);
    m.contracts[name].explorer = 'https://stellar.expert/explorer/testnet/contract/' + m.contracts[name].id;
  }
  const escrow = m.contracts.escrow.id;
  const token = addressValue(m.asset.contract);
  for (const name of ['nft_pool', 'revenue_pool']) {
    await f.call(a.governance, m.contracts[name].id, 'initialize', [
      addressValue(a.governance), addressValue(a.eligibility_authority), token,
      addressValue(escrow), ...deploymentProof(m.contracts[name].id, a.deployer,
        Buffer.from(m.contracts[name].salt, 'hex')),
    ], 'initialize_usdc_' + name);
  }
  await f.call(a.governance, escrow, 'initialize', [
    addressValue(a.governance), addressValue(a.settlement), token,
    addressValue(a.service_treasury), addressValue(m.contracts.nft_pool.id),
    addressValue(m.contracts.revenue_pool.id),
    ...deploymentProof(escrow, a.deployer, Buffer.from(m.contracts.escrow.salt, 'hex')),
  ], 'initialize_usdc_escrow');
  m.evidence.deployment_bindings = await verifyDeploymentBindings(m, f.view);
  await configureAutomation();
  assert.equal(BigInt(await f.view(escrow, 'total_liability')), 0n);
  for (const name of ['nft_pool', 'revenue_pool']) {
    assert.equal(BigInt(await f.view(m.contracts[name].id, 'available_balance')), 0n);
  }
  m.status = 'attested_usdc_deployed_waiting_for_circle_faucet';
  m.faucet_wait_expires_at = new Date(Date.now() + 60 * 60_000).toISOString();
  await f.save();
  console.log('FRESH_USDC_DEPLOYMENT_READY ' + f.path);
  console.log('ESCROW ' + escrow);
  console.log('FAUCET_STELLAR_TESTNET_USDC_TO ' + a.sponsor);
  console.log('Free Circle testnet USDC only; no real funds. Waiting at most 60 minutes.');
  const deadline = Date.now() + 60 * 60_000;
  while (await f.usdcBalance(a.sponsor) < 21_000_101n) {
    if (Date.now() > deadline) throw new Error('Faucet wait expired; no lifecycle payment sent');
    await sleep(30_000);
  }
  const amount = 10_000_000n;
  await fund(1, amount);
  await activate(1);
  const before = await f.balances();
  await f.reject(a.sponsor, escrow, 'refund_expired', [u64(1)], 9, 'early_refund_rejected');
  const unsigned = await f.prepare(a.settlement, f.invoke(escrow, 'complete', [u64(1), u32(1)]));
  const single = TransactionBuilder.fromXDR(unsigned.toXDR(), NETWORK);
  single.sign(members[0]);
  await submitPrepared(single, 'one_disposable_member_rejected', m, f.path, { expectBadAuth: true });
  assert.deepEqual(await f.balances(), before, 'Rejected completion must not move funds');
  unsigned.sign(members[0], members[1]);
  await submitPrepared(unsigned, 'two_disposable_members_complete_usdc_campaign_1', m, f.path);
  const after = await f.balances();
  m.evidence.campaign_1 = {
    before, after, expected_deltas: assertSplit(before, after, amount),
    status: await f.view(escrow, 'get_campaign', [u64(1)]),
    signer_scope: 'Automated disposable fixtures, not designated members',
    same_transaction_hash_for_one_and_two_signature_attempts: single.hash().equals(unsigned.hash()),
  };
  assert.equal(BigInt(await f.view(escrow, 'total_liability')), 0n);
  await f.reject(a.settlement, escrow, 'complete', [u64(1), u32(1)], 6, 'completed_campaign_replay_rejected');
  await allocateAndClaim('nft_pool', 500_000n);
  await allocateAndClaim('revenue_pool', 200_000n);
  m.evidence.pool_allocations_and_recipient_claims = 'confirmed; completed-manifest replay rejected';

  await fund(4, 101n);
  await activate(4);
  const roundingBefore = await f.balances();
  await f.call(a.settlement, escrow, 'complete', [u64(4), u32(1)], 'complete_101_base_unit_rounding_campaign');
  const roundingAfter = await f.balances();
  m.evidence.rounding = { before: roundingBefore, after: roundingAfter,
    expected_deltas: assertSplit(roundingBefore, roundingAfter, 101n) };
  // Separate human-member proof and new-USDC expiry fixture. Historical XLM IDs untouched.
  await fund(2, amount);
  await activate(2);
  await fund(3, 1_000_000n);
  await f.reject(a.sponsor, escrow, 'refund_expired', [u64(3)], 9, 'new_usdc_unactivated_early_refund_rejected');
  m.evidence.new_usdc_expiry_fixture = await f.view(escrow, 'get_campaign', [u64(3)]);
  assert.equal(BigInt(await f.view(escrow, 'total_liability')), 11_000_000n);
  const handoffOps = [];
  for (const role of ['governance', 'settlement']) {
    for (const publicKey of Object.values(m.designated_members)) handoffOps.push(Operation.setOptions({
      source: a[role], signer: { ed25519PublicKey: publicKey, weight: 1 },
    }));
    for (const key of members) handoffOps.push(Operation.setOptions({
      source: a[role], signer: { ed25519PublicKey: key.publicKey(), weight: 0 },
    }));
  }
  const handoff = await basicTransaction(a.deployer, handoffOps);
  handoff.sign(keys.deployer, members[0], members[1]);
  await submitPrepared(handoff, 'fresh_roles_handoff_to_ayoub_zyad_king', m, f.path);
  m.evidence.designated_role_accounts = {};
  for (const role of ['governance', 'settlement']) {
    const account = await horizon.loadAccount(a[role]);
    assertQuorum(account, Object.values(m.designated_members));
    m.evidence.designated_role_accounts[role] = {
      address: account.account_id, thresholds: account.thresholds, signers: account.signers,
    };
  }
  const request = await f.prepare(a.settlement, f.invoke(escrow, 'complete', [u64(2), u32(1)]), 86400);
  const data = request.toEnvelope().v1().tx().ext().sorobanData();
  const resource = data.resources();
  for (const field of ['instructions', 'diskReadBytes', 'readBytes', 'writeBytes']) {
    if (typeof resource[field] === 'function') resource[field](Math.min(4_000_000_000, resource[field]() * 4));
  }
  data.resourceFee(xdr.Int64.fromString(String(feeCap)));
  const envelope = request.toEnvelope();
  envelope.v1().tx().ext(new xdr.TransactionExt(1, data));
  envelope.v1().tx().fee(Number(feeCap + 10000n));
  const frozen = TransactionBuilder.fromXDR(envelope.toXDR('base64'), NETWORK);
  const check = await server.simulateTransaction(frozen, { authMode: 'enforce' });
  assert.ok(rpc.Api.isSimulationSuccess(check), 'Frozen member request must remain executable');
  assert.ok(BigInt(check.minResourceFee) <= feeCap);
  const needed = check.transactionData.build().resources();
  for (const field of ['instructions', 'diskReadBytes', 'readBytes', 'writeBytes']) {
    if (typeof resource[field] === 'function') assert.ok(resource[field]() >= needed[field]());
  }
  m.approval = {
    status: 'requires_two_designated_member_wallet_signatures',
    campaign_id: 2, amount_base_units: String(amount), asset: 'Circle testnet USDC',
    source: a.settlement, hash: frozen.hash().toString('hex'), unsigned_xdr: frozen.toXDR(),
    signature_deadline_utc: new Date(Number(frozen.timeBounds.maxTime) * 1000).toISOString(),
    signing_method: 'Signing-only; do not re-simulate or alter transaction data.',
  };
  await writeFile(resolve(out, 'complete-usdc-campaign-2.xdr.txt'), frozen.toXDR() + '\n');
  m.status = 'automated_usdc_lifecycle_passed; designated_member_release_and_expiry_refund_pending';
  await f.save();
  console.log('AUTOMATED_USDC_LIFECYCLE_PASSED ' + f.path);
  console.log('DESIGNATED_MEMBER_SIGNATURES_REQUIRED ' + resolve(out, 'complete-usdc-campaign-2.xdr.txt'));
} catch (error) {
  m.failure = error.message;
  m.status = 'incomplete; inspect persisted receipts before retrying; no acceptance claimed';
  await f.save();
  throw error;
}
