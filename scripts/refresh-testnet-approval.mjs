import { writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Contract, TransactionBuilder, nativeToScVal, scValToNative, xdr } from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, horizon, loadJson, validateSigners, assertQuorum, basicTransaction, saveManifest,
} from './testnet-common.mjs';

if (process.argv[2] !== '--refresh-testnet-approval') {
  throw new Error('Explicit --refresh-testnet-approval required');
}
const OUT = resolve(dirname(fileURLToPath(import.meta.url)), '../evidence/testnet-escrow-v2');
const PATH = resolve(OUT, 'manifest.json');
const manifest = await loadJson(PATH);
const keys = validateSigners(manifest);
if (manifest.approval.status === 'confirmed' || manifest.transactions.some(entry =>
  entry.label === 'two_of_three_completion_confirmed' && entry.status === 'confirmed_success')) {
  throw new Error('Cannot refresh a completed campaign');
}
for (const role of ['governance', 'settlement']) {
  assertQuorum(await horizon.loadAccount(manifest.accounts[role]), keys);
}
const contract = new Contract(manifest.contracts.escrow.id);
const campaign = nativeToScVal(1, { type: 'u64' });
const query = await basicTransaction(manifest.accounts.settlement, [contract.call('get_campaign', campaign)]);
const state = await server.simulateTransaction(query);
if (!rpc.Api.isSimulationSuccess(state) ||
    scValToNative(state.result.retval).status[0] !== 'Active') {
  throw new Error('Campaign is no longer active');
}
const completion = await basicTransaction(
  manifest.accounts.settlement, [
    contract.call('complete', campaign, nativeToScVal(1, { type: 'u32' })),
  ], 86400);
const recorded = await server.simulateTransaction(completion);
if (!rpc.Api.isSimulationSuccess(recorded)) throw new Error('Completion preparation failed');
const prepared = rpc.assembleTransaction(completion, recorded).build();
const data = recorded.transactionData.build();
const resources = data.resources();
// Testnet fixture only: reserve headroom for multisig verification and rent
// changes during collection. The fee payer is the free-funded role account.
for (const field of ['instructions', 'diskReadBytes', 'readBytes', 'writeBytes']) {
  if (typeof resources[field] === 'function') resources[field](resources[field]() * 4);
}
const feeCap = 100_000_000n; // 10 free test XLM, never a production fee policy.
data.resourceFee(xdr.Int64.fromString(String(feeCap)));
const envelope = prepared.toEnvelope();
envelope.v1().tx().ext(new xdr.TransactionExt(1, data));
envelope.v1().tx().fee(Number(feeCap + 10_000n));
const transaction = TransactionBuilder.fromXDR(envelope.toXDR('base64'), NETWORK);
const checked = await server.simulateTransaction(transaction, { authMode: 'enforce' });
if (!rpc.Api.isSimulationSuccess(checked) || BigInt(checked.minResourceFee) > feeCap) {
  throw new Error('Frozen completion does not cover the latest resource estimate');
}
const needed = checked.transactionData.build().resources();
for (const field of ['instructions', 'diskReadBytes', 'readBytes', 'writeBytes']) {
  if (typeof resources[field] === 'function' && resources[field]() < needed[field]()) {
    throw new Error('Frozen resource limit is insufficient: ' + field);
  }
}
if (transaction.toEnvelope().v1().tx().ext().switch() !== 1) {
  throw new Error('Serialized approval lost Soroban transaction data');
}
manifest.approval_history ??= [];
manifest.approval_history.push(manifest.approval);
manifest.approval = {
  ...manifest.approval, status: 'requires_two_member_wallet_signatures',
  hash: transaction.hash().toString('hex'), unsigned_xdr: transaction.toXDR(),
  signature_deadline_utc: new Date(Number(transaction.timeBounds.maxTime) * 1000).toISOString(),
  signing_method: 'Freighter official signing-only playground; do not re-simulate',
  fee_cap_test_xlm: '10.001',
};
await saveManifest(PATH, manifest);
await writeFile(resolve(OUT, 'complete-campaign-1.xdr.txt'), transaction.toXDR() + '\n');
console.log('Approval refreshed; previously collected signatures are not valid for this new envelope.');