// Owner-authorized original Campaign 2 refund. No mainnet or configured keys.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { access, readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  Address, Asset, Contract, Keypair, Networks, nativeToScVal, scValToNative, xdr,
} from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, horizon, activate, basicTransaction, saveManifest, submitPrepared,
} from './testnet-common.mjs';

assert.deepEqual(process.argv.slice(2), ['--execute-testnet'],
  'Explicit --execute-testnet is required');
assert.equal(NETWORK, Networks.TESTNET);
assert.equal((await server.getNetwork()).passphrase, Networks.TESTNET);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = resolve(root, 'evidence/original-30-day-xlm-refund/manifest.json');
try {
  await access(output);
  throw new Error('Evidence already exists. Inspect its signed hash; never submit a new transaction blindly.');
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
}
const original = JSON.parse(await readFile(
  resolve(root, 'evidence/testnet-escrow-v2/manifest.json'), 'utf8'));
const escrow = 'CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ';
const sponsor = 'GBAFTNVGOI6JBJH5TKYDIQGKFFKP2YZIHAPKQMHNML5ELEMHLGCUIEQG';
const token = Asset.native().contractId(Networks.TESTNET);
const amount = 10_000_000n;
const expiry = 1793709977n;
const u64 = n => nativeToScVal(n, { type: 'u64' });
const keyFor = key => xdr.LedgerKey.contractData(new xdr.LedgerKeyContractData({
  contract: new Address(escrow).toScAddress(),
  key, durability: xdr.ContractDataDurability.persistent(),
}));
async function campaign(id) {
  const result = await server.getLedgerEntries(keyFor(xdr.ScVal.scvVec([
    nativeToScVal('Campaign', { type: 'symbol' }), u64(id),
  ])));
  assert.equal(result.entries.length, 1, `Campaign ${id} missing or archived`);
  return scValToNative(result.entries[0].val.contractData().val());
}
async function instance() {
  const result = await server.getLedgerEntries(keyFor(xdr.ScVal.scvLedgerKeyContractInstance()));
  assert.equal(result.entries.length, 1, 'Escrow instance missing or archived');
  return Object.fromEntries(result.entries[0].val.contractData().val().instance().storage()
    .map(entry => [scValToNative(entry.key())[0], scValToNative(entry.val())]));
}
function normalized(value) {
  return JSON.parse(JSON.stringify(value, (_, v) => typeof v === 'bigint' ? String(v) : v));
}
const wasmHash = createHash('sha256')
  .update(await server.getContractWasmByContractId(escrow)).digest('hex');
assert.equal(wasmHash, original.wasm_sha256.campaign_escrow, 'Original deployed WASM changed');
const beforeCampaign = await campaign(2);
assert.deepEqual(beforeCampaign.status, ['Funded']);
assert.equal(beforeCampaign.sponsor, sponsor);
assert.equal(BigInt(beforeCampaign.amount), amount);
assert.equal(BigInt(beforeCampaign.expires_at), expiry);
assert.equal(BigInt(beforeCampaign.expires_at) - BigInt(beforeCampaign.funded_at), 30n * 86400n);
const caller = Keypair.random(); // Ephemeral, never persisted or printed.
await activate(caller.publicKey()); // Free Testnet Friendbot only.
async function balance(address) {
  const tx = await basicTransaction(caller.publicKey(), [
    new Contract(token).call('balance', new Address(address).toScVal()),
  ]);
  const sim = await server.simulateTransaction(tx);
  assert.ok(rpc.Api.isSimulationSuccess(sim), sim.error);
  return BigInt(scValToNative(sim.result.retval));
}
const beforeInstance = await instance();
assert.equal(beforeInstance.Token, token);
const before = {
  sponsor: await balance(sponsor), escrow: await balance(escrow),
  liability: BigInt(beforeInstance.TotalLiability),
};
assert.equal(before.escrow, amount);
assert.equal(before.liability, amount);
const otherCampaigns = Object.fromEntries(await Promise.all([1, 3, 4]
  .map(async id => [String(id), normalized(await campaign(id))])));
const manifest = {
  name: 'Original unmodified 30-day XLM Campaign 2 refund',
  network: 'testnet', network_passphrase: Networks.TESTNET,
  escrow, token, campaign_id: 2, sponsor, amount_base_units: String(amount),
  amount_xlm: '1', deployed_wasm_sha256: wasmHash,
  eligible_since_utc: new Date(Number(expiry) * 1000).toISOString(),
  created_at: new Date().toISOString(), caller: caller.publicKey(),
  custody: 'Temporary free-Testnet caller; no existing signing keys or mainnet funds',
  status: 'preflight_verified', campaign_before: normalized(beforeCampaign),
  before: normalized(before), other_campaigns_before: otherCampaigns,
  transactions: [],
};
await saveManifest(output, manifest);
const raw = await basicTransaction(caller.publicKey(), [
  new Contract(escrow).call('refund_expired', u64(2)),
]);
const simulation = await server.simulateTransaction(raw);
assert.ok(rpc.Api.isSimulationSuccess(simulation), simulation.error);
assert.ok(simulation.result.auth.every(entry =>
  entry.credentials().switch().name === 'sorobanCredentialsSourceAccount'),
'Refund unexpectedly requires another signing authority');
raw.operations[0].auth = simulation.result.auth;
const prepared = rpc.assembleTransaction(raw, simulation).build();
assert.ok(BigInt(prepared.fee) <= 100_000_000n, 'Fee exceeds 10 free test XLM cap');
prepared.sign(caller);
const confirmed = await submitPrepared(prepared,
  'original_campaign_2_real_30_day_xlm_refund', manifest, output);
const afterCampaign = await campaign(2);
const afterInstance = await instance();
const after = {
  sponsor: await balance(sponsor), escrow: await balance(escrow),
  liability: BigInt(afterInstance.TotalLiability),
};
assert.deepEqual(afterCampaign.status, ['Refunded']);
assert.equal(after.sponsor - before.sponsor, amount);
assert.equal(before.escrow - after.escrow, amount);
assert.equal(before.liability - after.liability, amount);
assert.equal(after.escrow, 0n);
assert.equal(after.liability, 0n);
for (const [key, value] of Object.entries(beforeInstance)) {
  if (key !== 'TotalLiability') assert.deepEqual(
    normalized(afterInstance[key]), normalized(value), `Configuration ${key} changed`);
}
for (const id of [1, 3, 4]) assert.deepEqual(
  normalized(await campaign(id)), otherCampaigns[String(id)], `Campaign ${id} changed`);
const horizonReceipt = await horizon.transactions().transaction(prepared.hash().toString('hex')).call();
assert.equal(horizonReceipt.successful, true);
manifest.campaign_after = normalized(afterCampaign);
manifest.after = normalized(after);
manifest.verification = {
  sponsor_refund_base_units: String(after.sponsor - before.sponsor),
  escrow_decrease_base_units: String(before.escrow - after.escrow),
  liability_decrease_base_units: String(before.liability - after.liability),
  other_campaigns_unchanged: true, escrow_configuration_unchanged: true,
};
manifest.receipt = {
  hash: horizonReceipt.hash, successful: true, ledger: confirmed.ledger,
  created_at: horizonReceipt.created_at, fee_charged: horizonReceipt.fee_charged,
  result_xdr: horizonReceipt.result_xdr,
  result_meta_xdr: confirmed.resultMetaXdr?.toXDR('base64'),
};
manifest.status = 'original_30_day_refund_confirmed_and_reconciled';
manifest.completed_at = new Date().toISOString();
await saveManifest(output, manifest);
console.log(JSON.stringify({
  hash: horizonReceipt.hash, ledger: confirmed.ledger,
  confirmed_at: horizonReceipt.created_at, refunded_xlm: '1',
  campaign_status: 'Refunded', escrow_balance: String(after.escrow),
  escrow_liability: String(after.liability), evidence: output,
}));
