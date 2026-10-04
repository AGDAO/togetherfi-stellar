import { writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Contract, nativeToScVal, scValToNative } from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  server, horizon, loadJson, validateSigners, assertQuorum, basicTransaction, saveManifest,
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
const transaction = await server.prepareTransaction(await basicTransaction(
  manifest.accounts.settlement, [
    contract.call('complete', campaign, nativeToScVal(1, { type: 'u32' })),
  ], 86400));
manifest.approval_history ??= [];
manifest.approval_history.push(manifest.approval);
manifest.approval = {
  ...manifest.approval, status: 'requires_two_member_wallet_signatures',
  hash: transaction.hash().toString('hex'), unsigned_xdr: transaction.toXDR(),
  signature_deadline_utc: new Date(Number(transaction.timeBounds.maxTime) * 1000).toISOString(),
};
await saveManifest(PATH, manifest);
await writeFile(resolve(OUT, 'complete-campaign-1.xdr.txt'), transaction.toXDR() + '\n');
console.log('Approval refreshed; previously collected signatures are not valid for this new envelope.');