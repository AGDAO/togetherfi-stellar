// No mainnet, real funds, member seeds, escrow mutation, or retained fixture seeds.
import { access } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Contract, Keypair, Operation, nativeToScVal, scValToNative } from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, activate, basicTransaction, addressValue, saveManifest,
  submitPrepared, sleep,
} from './testnet-common.mjs';
import {
  CIRCLE_SOURCE, STELLAR_USDC, testnetUsdcAsset, assertUsdcTransferDeltas,
} from './stellar-usdc.mjs';

if (process.argv.length !== 3 || process.argv[2] !== '--execute-testnet') {
  throw new Error('Only --execute-testnet is supported');
}
const asset = testnetUsdcAsset(NETWORK);
const network = await server.getNetwork();
if (network.passphrase !== NETWORK) throw new Error('RPC is not Stellar Testnet');
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const PATH = resolve(ROOT, 'evidence/testnet-usdc-wallet/manifest.json');
try {
  await access(PATH);
  throw new Error('Existing test evidence must not be overwritten');
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
}
const sponsor = Keypair.random();
const recipient = Keypair.random();
const manifest = {
  network: 'testnet', created_at: new Date().toISOString(),
  status: 'preparing_trustlines', asset: STELLAR_USDC.testnet,
  identity_source: CIRCLE_SOURCE,
  accounts: { sponsor: sponsor.publicKey(), recipient: recipient.publicKey() },
  amount_base_units: '100000', amount_usdc: '0.0100000', transactions: [],
  scope: 'Circle testnet USDC SAC wallet transfer only; not escrow, quorum or production acceptance.',
  fixture_keys: 'Ephemeral process-memory keys; no member keys read and no seeds retained.',
};
await saveManifest(PATH, manifest);

async function view(method, args = []) {
  const raw = await basicTransaction(sponsor.publicKey(), [
    new Contract(STELLAR_USDC.testnet.sac).call(method, ...args),
  ]);
  const result = await server.simulateTransaction(raw);
  if (!rpc.Api.isSimulationSuccess(result)) throw new Error('USDC read failed: ' + result.error);
  return scValToNative(result.result.retval);
}
async function balances() {
  return {
    sender: BigInt(await view('balance', [addressValue(sponsor.publicKey())])),
    recipient: BigInt(await view('balance', [addressValue(recipient.publicKey())])),
  };
}

try {
  await activate(sponsor.publicKey());
  await activate(recipient.publicKey());
  const trustlines = await basicTransaction(sponsor.publicKey(), [
    Operation.changeTrust({ asset, limit: '20', source: sponsor.publicKey() }),
    Operation.changeTrust({ asset, limit: '20', source: recipient.publicKey() }),
  ]);
  trustlines.sign(sponsor, recipient);
  await submitPrepared(trustlines, 'create_circle_usdc_trustlines', manifest, PATH);
  manifest.observed_decimals = Number(await view('decimals'));
  manifest.observed_symbol = await view('symbol');
  if (manifest.observed_decimals !== 7 || manifest.observed_symbol !== 'USDC') {
    throw new Error('Unexpected SAC metadata');
  }
  manifest.status = 'waiting_for_human_circle_faucet';
  await saveManifest(PATH, manifest);
  console.log('WAITING_FOR_CIRCLE_FAUCET');
  console.log('Network: Stellar Testnet; token: USDC; faucet: https://faucet.circle.com/');
  console.log('Send free testnet USDC to: ' + sponsor.publicKey());
  console.log('Both USDC trustlines are confirmed. No real money is required.');
  const amount = 100_000n;
  const deadline = Date.now() + 30 * 60_000;
  let before;
  while (Date.now() < deadline) {
    before = await balances();
    if (before.sender >= amount) break;
    await sleep(30_000);
  }
  if (!before || before.sender < amount) {
    manifest.status = 'faucet_wait_expired_no_payment_sent';
    await saveManifest(PATH, manifest);
    throw new Error('Circle faucet funding did not arrive within 30 minutes');
  }
  const raw = await basicTransaction(sponsor.publicKey(), [
    new Contract(STELLAR_USDC.testnet.sac).call('transfer',
      addressValue(sponsor.publicKey()), addressValue(recipient.publicKey()),
      nativeToScVal(amount, { type: 'i128' })),
  ]);
  const simulation = await server.simulateTransaction(raw);
  if (!rpc.Api.isSimulationSuccess(simulation)) {
    throw new Error('USDC transfer simulation failed: ' + simulation.error);
  }
  const prepared = rpc.assembleTransaction(raw, simulation).build();
  if (BigInt(prepared.fee) > 100_000_000n) throw new Error('Testnet XLM fee exceeds 10 XLM cap');
  prepared.sign(sponsor);
  manifest.before = before;
  await submitPrepared(prepared, 'transfer_0_01_circle_testnet_usdc', manifest, PATH);
  const after = await balances();
  assertUsdcTransferDeltas(before, after, amount);
  manifest.after = after;
  manifest.status = 'confirmed_usdc_wallet_smoke_only';
  manifest.completed_at = new Date().toISOString();
  await saveManifest(PATH, manifest);
  console.log('SUCCESS: 0.0100000 Circle testnet USDC transferred; exact SAC deltas verified.');
} catch (error) {
  manifest.error = error.message;
  if (!manifest.status.startsWith('faucet_wait_expired')) manifest.status = 'failed_or_pending_review_manifest';
  await saveManifest(PATH, manifest);
  throw error;
}
