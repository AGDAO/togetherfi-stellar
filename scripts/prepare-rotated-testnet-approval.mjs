// A fresh free-XLM campaign on the existing testnet contracts, not a deployment.
// Disposable fixture keys exist only in memory; member keys are never read.
import assert from 'node:assert/strict';
import { access, rename, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  Asset, Contract, Keypair, Networks, TransactionBuilder,
  nativeToScVal, scValToNative, xdr,
} from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, horizon, loadJson, validateSigners, assertQuorum,
  basicTransaction, addressValue, activate, submitPrepared, saveManifest,
} from './testnet-common.mjs';
import { ZYAD_TESTNET_KEY } from './prepare-testnet-signer-rotation.mjs';

// Long-lived fixture entries incur testnet rent. Quote-based headroom is bounded,
// never applied to production and never taken from a member's personal wallet.
export function freeTestFeeCap(quotedFee) {
  assert.match(String(quotedFee), /^[0-9]+$/);
  const cap = BigInt(quotedFee) * 2n + 1_000_000n;
  assert.ok(cap <= 2_000_000_000n, 'Free-test-XLM fixture fee exceeds its safety limit');
  return cap;
}

export function validateRotatedEvidence(baseline, rotation) {
  assert.equal(baseline.network, 'testnet');
  assert.equal(rotation.network, 'testnet');
  assert.equal(rotation.status, 'confirmed_success');
  assert.match(rotation.hash, /^[a-f0-9]{64}$/);
  assert.ok(Number.isInteger(rotation.ledger) && rotation.ledger > 0);
  assert.equal(baseline.asset.contract, Asset.native().contractId(Networks.TESTNET));
  assert.equal(baseline.approval.status, 'confirmed', 'Preserve the completed historical campaign');
  const expected = { Ayoub: baseline.signers.Ayoub, Zyad: ZYAD_TESTNET_KEY, King: baseline.signers.King };
  assert.deepEqual(rotation.after_signers, expected);
  const keys = validateSigners({ network: 'testnet', signers: expected });
  const roles = [baseline.accounts.governance, baseline.accounts.settlement];
  assert.equal(new Set(roles).size, 2);
  assert.ok(roles.every(role => !keys.includes(role)));
  assert.equal(rotation.accounts.length, 2);
  for (const role of roles) {
    const account = rotation.accounts.find(item => item.account_id === role);
    assert.ok(account, 'Missing confirmed dedicated role account');
    assertQuorum(account, keys);
  }
  return keys;
}

async function main() {
  if (process.argv[2] !== '--prepare-testnet') {
    throw new Error('Explicit --prepare-testnet required; mainnet is unsupported');
  }
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  const out = resolve(root, 'evidence/testnet-zyad-king');
  const path = resolve(out, 'manifest.json');
  try {
    await access(path);
    if (!process.argv.includes('--archive-unfunded-attempt')) {
      throw new Error('Existing evidence must not be overwritten or repeated automatically');
    }
    const previous = await loadJson(path);
    assert.equal(previous.network, 'testnet');
    assert.equal(previous.failure, 'Unexpected free-test-XLM fixture fee');
    assert.equal(previous.transactions.length, 0,
      'Any persisted transaction must be reconciled; do not replace its evidence');
    assert.equal(previous.approval, undefined);
    await rename(path, resolve(out, 'preparation-unfunded-' + Date.now() + '.json'));
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  const baseline = await loadJson(resolve(root, 'evidence/testnet-escrow-v2/manifest.json'));
  const rotation = await loadJson(resolve(root, 'evidence/testnet-signer-rotation/confirmation.json'));
  const keys = validateRotatedEvidence(baseline, rotation);
  // Horizon SDK link hydration can replace the numeric ledger field with a method.
  const response = await fetch('https://horizon-testnet.stellar.org/transactions/' + rotation.hash,
    { signal: AbortSignal.timeout(20000) });
  if (!response.ok) throw new Error('Cannot verify rotation receipt: ' + response.status);
  const confirmed = await response.json();
  assert.equal(confirmed.hash, rotation.hash);
  assert.equal(confirmed.successful, true);
  assert.equal(confirmed.ledger, rotation.ledger);
  for (const role of ['governance', 'settlement']) {
    assertQuorum(await horizon.loadAccount(baseline.accounts[role]), keys);
  }
  const fixture = Object.fromEntries(['sponsor', 'creator', 'referrer']
    .map(name => [name, Keypair.random()]));
  const manifest = {
    network: 'testnet', created_at: new Date().toISOString(),
    status: 'new_roster_fixture_preparation_in_progress',
    baseline_evidence: '../testnet-escrow-v2/manifest.json',
    rotation_evidence: '../testnet-signer-rotation/confirmation.json',
    signers: rotation.after_signers, policy: baseline.policy,
    contracts: baseline.contracts, asset: baseline.asset,
    accounts: { ...baseline.accounts,
      ...Object.fromEntries(Object.entries(fixture).map(([name, key]) => [name, key.publicKey()])) },
    campaign_id: 3, transactions: [], evidence: {},
    fixture_custody: 'Free testnet XLM only. Disposable sponsor/recipient fixtures; no production custody or member keys.',
    requested_release_signers: ['Zyad', 'King'],
    review: 'Fresh human reviews pending; signatures are not code-review approvals.',
  };
  const escrow = new Contract(manifest.contracts.escrow.id);
  async function view(contract, method, args = []) {
    const raw = await basicTransaction(manifest.accounts.settlement, [contract.call(method, ...args)]);
    const result = await server.simulateTransaction(raw);
    if (!rpc.Api.isSimulationSuccess(result)) throw new Error('Read failed: ' + method);
    return scValToNative(result.result.retval);
  }
  assert.equal(await view(escrow, 'governance'), manifest.accounts.governance);
  assert.equal(await view(escrow, 'settlement_authority'), manifest.accounts.settlement);
  await saveManifest(path, manifest);
  try {
    for (const key of Object.values(fixture)) await activate(key.publicKey());
    async function sponsorCall(method, args, label) {
      const raw = await basicTransaction(fixture.sponsor.publicKey(), [escrow.call(method, ...args)]);
      const simulation = await server.simulateTransaction(raw);
      if (!rpc.Api.isSimulationSuccess(simulation)) {
        throw new Error(label + ': ' + simulation.error);
      }
      assert.ok(simulation.result.auth.every(entry =>
        entry.credentials().switch().name === 'sorobanCredentialsSourceAccount'));
      const prepared = rpc.assembleTransaction(raw, simulation).build();
      assert.ok(Number(prepared.fee) <= 2_000_000_000,
        'Free-test-XLM fixture fee exceeds its safety limit');
      prepared.sign(fixture.sponsor);
      return submitPrepared(prepared, label, manifest, path);
    }
    const campaign = nativeToScVal(manifest.campaign_id, { type: 'u64' });
    await sponsorCall('fund', [
      campaign, addressValue(manifest.accounts.sponsor), nativeToScVal(10_000_000n, { type: 'i128' }),
    ], 'fund_campaign_3_for_new_roster');
    await sponsorCall('activate', [
      campaign, addressValue(manifest.accounts.creator), addressValue(manifest.accounts.referrer),
      nativeToScVal(Buffer.from('native-testnet-zyad-king-quorum')),
    ], 'activate_campaign_3_for_new_roster');
    manifest.evidence.campaign_before_completion = await view(escrow, 'get_campaign', [campaign]);
    assert.equal(manifest.evidence.campaign_before_completion.status[0], 'Active');
    const destinations = {
      creator: manifest.accounts.creator, service: manifest.accounts.service_treasury,
      nft_pool: manifest.contracts.nft_pool.id, revenue_pool: manifest.contracts.revenue_pool.id,
      referrer: manifest.accounts.referrer, escrow: manifest.contracts.escrow.id,
    };
    const token = new Contract(manifest.asset.contract);
    manifest.evidence.balances_before_completion = Object.fromEntries(
      await Promise.all(Object.entries(destinations).map(async ([name, address]) =>
        [name, String(await view(token, 'balance', [addressValue(address)]))])));
    const raw = await basicTransaction(manifest.accounts.settlement, [
      escrow.call('complete', campaign, nativeToScVal(1, { type: 'u32' })),
    ], 86400);
    const simulation = await server.simulateTransaction(raw);
    if (!rpc.Api.isSimulationSuccess(simulation)) throw new Error('Completion preparation failed');
    assert.ok(simulation.result.auth.every(entry =>
      entry.credentials().switch().name === 'sorobanCredentialsSourceAccount'));
    const assembled = rpc.assembleTransaction(raw, simulation).build();
    const data = simulation.transactionData.build();
    const resources = data.resources();
    for (const field of ['instructions', 'diskReadBytes', 'readBytes', 'writeBytes']) {
      if (typeof resources[field] === 'function') resources[field](resources[field]() * 4);
    }
    const feeCap = freeTestFeeCap(simulation.minResourceFee);
    data.resourceFee(xdr.Int64.fromString(String(feeCap)));
    const envelope = assembled.toEnvelope();
    envelope.v1().tx().ext(new xdr.TransactionExt(1, data));
    envelope.v1().tx().fee(Number(feeCap + 10_000n));
    const frozen = TransactionBuilder.fromXDR(envelope.toXDR('base64'), NETWORK);
    const checked = await server.simulateTransaction(frozen, { authMode: 'enforce' });
    if (!rpc.Api.isSimulationSuccess(checked) || BigInt(checked.minResourceFee) > feeCap) {
      throw new Error('Frozen resource allowance does not cover the latest simulation');
    }
    const needed = checked.transactionData.build().resources();
    for (const field of ['instructions', 'diskReadBytes', 'readBytes', 'writeBytes']) {
      if (typeof resources[field] === 'function') assert.ok(resources[field]() >= needed[field]());
    }
    assert.equal(frozen.toEnvelope().v1().tx().ext().switch(), 1);
    assert.equal(frozen.signatures.length, 0);
    manifest.approval = {
      status: 'requires_zyad_and_king_wallet_signatures', hash: frozen.hash().toString('hex'),
      unsigned_xdr: frozen.toXDR(), source: manifest.accounts.settlement,
      campaign_id: 3, operation_version: 1, amount_base_units: '10000000',
      expected_split: '82/10/5/2/1',
      quoted_resource_fee_stroops: String(simulation.minResourceFee),
      fee_cap_test_xlm: String(Number(feeCap + 10_000n) / 10_000_000),
      signature_deadline_utc: new Date(Number(frozen.timeBounds.maxTime) * 1000).toISOString(),
      signing_method: 'Freighter signing-only; sign this identical frozen envelope separately; do not re-simulate',
    };
    manifest.status = 'campaign_3_active; zyad_king_completion_and_one_signature_rejection_pending';
    await saveManifest(path, manifest);
    await writeFile(resolve(out, 'complete-campaign-3.xdr.txt'), frozen.toXDR() + '\n', { flag: 'wx' });
    console.log(JSON.stringify({
      network: 'testnet', status: manifest.status, approval_hash: manifest.approval.hash,
      deadline_utc: manifest.approval.signature_deadline_utc,
      fee_cap_test_xlm: manifest.approval.fee_cap_test_xlm,
      amount_test_xlm: 1, release_submitted: false,
    }, null, 2));
  } catch (error) {
    manifest.status = 'incomplete; reconcile existing evidence before taking further action';
    manifest.failure = error.message;
    await saveManifest(path, manifest);
    throw error;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
