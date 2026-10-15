// Consume wallet-signed public XDR; no member private keys are read.
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Contract, TransactionBuilder, nativeToScVal, scValToNative } from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, horizon, loadJson, validateSigners, assertQuorum,
  approvedSignatureCount, submitPrepared, saveManifest, basicTransaction, addressValue,
} from './testnet-common.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const PATH = resolve(ROOT, 'evidence/testnet-escrow-v2/manifest.json');
const input = process.argv[2];
if (!input) throw new Error('Path to wallet-signed testnet XDR required');
const manifest = await loadJson(PATH);
const keys = validateSigners(manifest);
if (manifest.approval?.status === 'confirmed') throw new Error('Campaign already completed');
if (!manifest.approval?.unsigned_xdr) throw new Error('No prepared completion approval exists');
const signed = TransactionBuilder.fromXDR((await readFile(input, 'utf8')).trim(), NETWORK);
if (signed.hash().toString('hex') !== manifest.approval.hash ||
    signed.source !== manifest.accounts.settlement || signed.operations.length !== 1) {
  throw new Error('Signed transaction differs from the exact approved testnet completion');
}
if (approvedSignatureCount(signed, keys) < 2) {
  throw new Error('Two distinct selected signer signatures required; do not submit yet');
}
for (const role of ['governance', 'settlement']) {
  assertQuorum(await horizon.loadAccount(manifest.accounts[role]), keys);
}
const prior = manifest.transactions.find(entry =>
  entry.label === 'two_of_three_completion_confirmed' &&
  entry.hash === manifest.approval.hash);
let result;
if (prior) {
  result = await server.getTransaction(prior.hash);
  if (result.status === 'FAILED') throw new Error('Previously broadcast completion failed');
  if (result.status !== 'SUCCESS') {
    if (prior.status === 'confirmed_success') {
      throw new Error('Previously confirmed completion unavailable; do not resubmit');
    }
    const frozen = TransactionBuilder.fromXDR(prior.signed_xdr, NETWORK);
    if (frozen.hash().toString('hex') !== manifest.approval.hash ||
        approvedSignatureCount(frozen, keys) < 2) {
      throw new Error('Persisted completion envelope failed validation');
    }
    // A crash/unknown response must replay the exact saved signed bytes.
    result = await submitPrepared(frozen, 'two_of_three_completion_confirmed', manifest, PATH);
  }
} else {
  // Preserve an actual member's single signature to prove 1-of-3 rejection.
  const one = TransactionBuilder.fromXDR(signed.toXDR(), NETWORK);
  while (one.signatures.length > 1) one.signatures.pop();
  if (approvedSignatureCount(one, keys) !== 1) {
    throw new Error('First signature must belong to a selected member');
  }
  const rejected = await submitPrepared(one, 'one_of_three_member_completion_rejected',
    manifest, PATH, { expectBadAuth: true });
  manifest.evidence.one_of_three_rejection = {
    reason: rejected.reason, type: 'RPC ingestion rejection, not a confirmed ledger transaction',
  };
  result = await submitPrepared(signed, 'two_of_three_completion_confirmed', manifest, PATH);
}
const expected = { creator: 8_200_000n, service: 1_000_000n,
  nft_pool: 500_000n, revenue_pool: 200_000n, referrer: 100_000n, escrow: -10_000_000n };
const addresses = {
  creator: manifest.accounts.creator, service: manifest.accounts.service_treasury,
  nft_pool: manifest.contracts.nft_pool.id, revenue_pool: manifest.contracts.revenue_pool.id,
  referrer: manifest.accounts.referrer, escrow: manifest.contracts.escrow.id,
};
const after = {};
for (const [name, address] of Object.entries(addresses)) {
  const raw = await basicTransaction(manifest.accounts.settlement, [
    new Contract(manifest.asset.contract).call('balance', addressValue(address)),
  ]);
  const simulation = await server.simulateTransaction(raw);
  if (!rpc.Api.isSimulationSuccess(simulation)) throw new Error('Balance read failed');
  after[name] = String(scValToNative(simulation.result.retval));
  const delta = BigInt(after[name]) - BigInt(manifest.evidence.balances_before_completion[name]);
  if (delta !== expected[name]) throw new Error('Payout delta mismatch for ' + name);
}
const raw = await basicTransaction(manifest.accounts.settlement, [
  new Contract(manifest.contracts.escrow.id).call('get_campaign', nativeToScVal(1, { type: 'u64' })),
]);
const simulation = await server.simulateTransaction(raw);
if (!rpc.Api.isSimulationSuccess(simulation)) throw new Error('Campaign read failed');
manifest.evidence.campaign_1_after_completion = scValToNative(simulation.result.retval);
if (manifest.evidence.campaign_1_after_completion.status[0] !== 'Completed') {
  throw new Error('Campaign is not in the expected terminal Completed state');
}
const counters = {};
for (const name of ['escrow', 'nft_pool', 'revenue_pool']) {
  const ledger = await server.getLedgerEntries(new Contract(manifest.contracts[name].id).getFootprint());
  if (ledger.entries.length !== 1) throw new Error('Contract instance unavailable: ' + name);
  counters[name] = Object.fromEntries(
    ledger.entries[0].val.contractData().val().instance().storage().map(entry => {
      const key = scValToNative(entry.key());
      return [Array.isArray(key) ? key[0] : key, scValToNative(entry.val())];
    }).filter(([key]) => ['TotalLiability', 'TotalCredited', 'TotalPaid'].includes(key)));
}
if (counters.escrow.TotalLiability !== 10_000_000n ||
    counters.nft_pool.TotalCredited !== 500_000n ||
    counters.revenue_pool.TotalCredited !== 200_000n) {
  throw new Error('Escrow liability or authenticated pool-credit counters do not reconcile');
}
manifest.evidence.accounting_after_completion = counters;
manifest.evidence.liability_reconciled = 'Only the funded 1-test-XLM expiry campaign remains owed';
manifest.evidence.balances_after_completion = after;
manifest.evidence.split_verified = '82/10/5/2/1, exact base-unit deltas';
manifest.approval.status = 'confirmed';
manifest.approval.completion_ledger = result.ledger;
manifest.status = 'quorum_completion_verified; real_30_day_refund_proof_pending';
await saveManifest(PATH, manifest);
console.log('TESTNET_QUORUM_COMPLETION_VERIFIED');