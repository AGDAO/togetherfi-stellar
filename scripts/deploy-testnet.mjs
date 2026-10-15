// Testnet fixtures only. Setup keys exist only in process memory, never in output.
import { readFile, writeFile, access } from 'node:fs/promises';
import { randomBytes, createHash } from 'node:crypto';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  Address, Asset, Contract, Keypair, Networks, Operation, authorizeEntry,
  nativeToScVal, scValToNative, TransactionBuilder,
} from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, horizon, addressValue, validateSigners, assertQuorum,
  contractId, saveManifest, activate, submitPrepared, basicTransaction,
} from './testnet-common.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const OUT = resolve(ROOT, 'evidence/testnet-escrow-v2');
const PATH = resolve(OUT, 'manifest.json');
if (process.argv[2] !== '--execute-testnet') {
  throw new Error('Explicit --execute-testnet required; mainnet is unsupported');
}
try { await access(PATH); throw new Error('Existing manifest must not be overwritten'); }
catch (error) { if (error.code !== 'ENOENT') throw error; }

const manifest = {
  network: 'testnet', created_at: new Date().toISOString(),
  status: 'bootstrap_in_progress', review: 'owner-only; no independent review claimed',
  signers: {
    Ayoub: 'GDGWL4MGDMHYLDKWTKOMH6RY2B4LR3A4GDUVGXS64LKPLSDTBL4PVBLE',
    Ace: 'GD57W22FBBREDJSEN2PDRLP5FYBZQ4QHO65KN6Z6U2ZP3I2OCOALJFTH',
    King: 'GARRSD6FDAUASB5UMPM6KV4UQZGWO47NBUY6F2GFC4V4UE6YOWRZDS5U',
  },
  policy: { quorum: 2, signer_weight: 1, master_weight: 0 },
  asset: {
    symbol: 'XLM', decimals: 7, fixture_only: true,
    contract: Asset.native().contractId(Networks.TESTNET),
    note: 'Free testnet XLM only; not final production asset or USDC.',
  },
  contracts: {}, accounts: {}, transactions: [], evidence: {},
};
const signerKeys = validateSigners(manifest);
// No member signing keys are generated or read. These are disposable setup/fixture keys.
const fixture = Object.fromEntries(
  ['deployer', 'governance', 'settlement', 'sponsor', 'creator', 'referrer']
    .map(name => [name, Keypair.random()]));
for (const [name, key] of Object.entries(fixture)) manifest.accounts[name] = key.publicKey();
manifest.accounts.service_treasury = fixture.deployer.publicKey();
manifest.accounts.eligibility_authority = fixture.deployer.publicKey();
manifest.fixture_custody = 'Sponsor, recipients, treasury and eligibility are test fixtures, not production custody.';
const salts = Object.fromEntries(['escrow', 'nft_pool', 'revenue_pool']
  .map(name => [name, randomBytes(32)]));
for (const [name, salt] of Object.entries(salts)) {
  manifest.contracts[name] = { id: contractId(fixture.deployer.publicKey(), salt), salt: salt.toString('hex') };
}
await saveManifest(PATH, manifest);

async function soroban(key, operation, label, additionalKeys = []) {
  const raw = await basicTransaction(key.publicKey(), [operation]);
  const simulation = await server.simulateTransaction(raw);
  if (!rpc.Api.isSimulationSuccess(simulation)) throw new Error(label + ': ' + simulation.error);
  const auth = await Promise.all(simulation.result.auth.map(async entry => {
    if (entry.credentials().switch().name === 'sorobanCredentialsSourceAccount') return entry;
    const address = Address.fromScAddress(entry.credentials().address().address()).toString();
    const signer = [key, ...additionalKeys].find(candidate => candidate.publicKey() === address);
    if (!signer) throw new Error('No fixture authorization available for ' + address);
    return authorizeEntry(entry, signer, simulation.latestLedger + 200, NETWORK);
  }));
  raw.operations[0].auth = auth;
  const prepared = rpc.assembleTransaction(raw, simulation).build();
  prepared.sign(key);
  return submitPrepared(prepared, label, manifest, PATH);
}
const invoke = (id, method, args) => new Contract(id).call(method, ...args);
async function view(id, method, args = []) {
  const tx = await basicTransaction(fixture.deployer.publicKey(), [invoke(id, method, args)]);
  const result = await server.simulateTransaction(tx);
  if (!rpc.Api.isSimulationSuccess(result)) throw new Error('Read failed ' + method + ': ' + result.error);
  return scValToNative(result.result.retval);
}
async function balances() {
  const destinations = {
    creator: manifest.accounts.creator, service: manifest.accounts.service_treasury,
    nft_pool: manifest.contracts.nft_pool.id, revenue_pool: manifest.contracts.revenue_pool.id,
    referrer: manifest.accounts.referrer, escrow: manifest.contracts.escrow.id,
  };
  return Object.fromEntries(await Promise.all(Object.entries(destinations).map(async ([name, id]) =>
    [name, String(await view(manifest.asset.contract, 'balance', [addressValue(id)]))])));
}
async function call(key, id, method, args, label, extras = []) {
  return soroban(key, invoke(id, method, args), label, extras);
}

try {
  for (const key of signerKeys) await activate(key);
  for (const key of Object.values(fixture)) await activate(key.publicKey());
  const packageSpecs = [
    ['campaign_escrow', '1cfbad51dc1932e81de857b4a22402f883584ba4c6b89388aff53929c2e6b0eb'],
    ['contributor_pool', '2f24a684373cfb2683912b1a85be0b8e4ac859fb64c15b14a52050579e99a865'],
  ];
  const wasmHashes = {};
  for (const [name, expected] of packageSpecs) {
    const wasm = await readFile(resolve(ROOT, 'contracts', name,
      'target/wasm32-unknown-unknown/release', name + '.wasm'));
    const actual = createHash('sha256').update(wasm).digest('hex');
    if (actual !== expected) throw new Error('WASM differs from tested input: ' + name);
    wasmHashes[name] = actual;
    await soroban(fixture.deployer, Operation.uploadContractWasm({ wasm }), 'upload_' + name);
  }
  manifest.wasm_sha256 = wasmHashes;
  for (const name of ['escrow', 'nft_pool', 'revenue_pool']) {
    const packageName = name === 'escrow' ? 'campaign_escrow' : 'contributor_pool';
    const result = await soroban(fixture.deployer, Operation.createCustomContract({
      address: new Address(fixture.deployer.publicKey()),
      wasmHash: Buffer.from(wasmHashes[packageName], 'hex'), salt: salts[name],
    }), 'deploy_' + name);
    if (scValToNative(result.returnValue) !== manifest.contracts[name].id) {
      throw new Error('Deployed contract ID mismatch');
    }
  }
  const gov = addressValue(manifest.accounts.governance);
  const token = addressValue(manifest.asset.contract);
  const escrow = manifest.contracts.escrow.id;
  for (const name of ['nft_pool', 'revenue_pool']) {
    await call(fixture.governance, manifest.contracts[name].id, 'initialize', [
      gov, addressValue(manifest.accounts.eligibility_authority), token, addressValue(escrow),
    ], 'initialize_' + name, [fixture.deployer]);
  }
  await call(fixture.governance, escrow, 'initialize', [
    gov, addressValue(manifest.accounts.settlement), token,
    addressValue(manifest.accounts.service_treasury),
    addressValue(manifest.contracts.nft_pool.id), addressValue(manifest.contracts.revenue_pool.id),
  ], 'initialize_escrow');
  // Detect a failed/mismatched initializer before handing off the role accounts.
  if (await view(escrow, 'governance') !== manifest.accounts.governance ||
      await view(escrow, 'settlement_authority') !== manifest.accounts.settlement) {
    throw new Error('Escrow role binding mismatch');
  }
  const options = [];
  for (const role of ['governance', 'settlement']) {
    for (const publicKey of signerKeys) options.push(Operation.setOptions({
      source: manifest.accounts[role], signer: { ed25519PublicKey: publicKey, weight: 1 },
    }));
    options.push(Operation.setOptions({
      source: manifest.accounts[role], masterWeight: 0,
      lowThreshold: 2, medThreshold: 2, highThreshold: 2,
    }));
  }
  const handoff = await basicTransaction(fixture.deployer.publicKey(), options);
  handoff.sign(fixture.deployer, fixture.governance, fixture.settlement);
  await submitPrepared(handoff, 'handoff_both_roles_to_2_of_3', manifest, PATH);
  manifest.evidence.multisig = {};
  for (const role of ['governance', 'settlement']) {
    const account = await horizon.loadAccount(manifest.accounts[role]);
    assertQuorum(account, signerKeys);
    manifest.evidence.multisig[role] = {
      address: account.account_id, thresholds: account.thresholds, signers: account.signers,
    };
  }
  const rotation = await server.prepareTransaction(await basicTransaction(
    manifest.accounts.governance, [
      invoke(escrow, 'propose_settlement', [addressValue(manifest.accounts.deployer)]),
    ]));
  rotation.sign(fixture.governance);
  const rotationRejected = await submitPrepared(rotation,
    'disabled_backend_key_role_rotation_rejected', manifest, PATH, { expectBadAuth: true });
  manifest.evidence.governance_backend_rejection = {
    reason: rotationRejected.reason, hash: rotationRejected.hash,
    type: 'RPC ingestion rejection, not a confirmed ledger transaction',
  };
  if (await view(escrow, 'settlement_authority') !== manifest.accounts.settlement) {
    throw new Error('Settlement role changed during rejected rotation');
  }
  const campaign = nativeToScVal(1, { type: 'u64' });
  const amount = nativeToScVal(10_000_000n, { type: 'i128' });
  await call(fixture.sponsor, escrow, 'fund', [
    campaign, addressValue(manifest.accounts.sponsor), amount,
  ], 'fund_campaign_1');
  await call(fixture.sponsor, escrow, 'activate', [
    campaign, addressValue(manifest.accounts.creator), addressValue(manifest.accounts.referrer),
    nativeToScVal(Buffer.from('escrow-v2-testnet-acceptance')),
  ], 'activate_campaign_1');
  // A separate campaign preserves the real 30-day refund boundary for later proof.
  await call(fixture.sponsor, escrow, 'fund', [
    nativeToScVal(2, { type: 'u64' }), addressValue(manifest.accounts.sponsor), amount,
  ], 'fund_campaign_2_for_real_expiry');
  manifest.evidence.campaign_1 = await view(escrow, 'get_campaign', [campaign]);
  manifest.evidence.campaign_2 = await view(escrow, 'get_campaign', [nativeToScVal(2, { type: 'u64' })]);
  manifest.evidence.balances_before_completion = await balances();
  const raw = await basicTransaction(manifest.accounts.settlement, [
    invoke(escrow, 'complete', [campaign, nativeToScVal(1, { type: 'u32' })]),
  ], 86400);
  const prepared = await server.prepareTransaction(raw);
  // This temporary master has weight zero after handoff. Submission must reject it.
  prepared.sign(fixture.settlement);
  const rejected = await submitPrepared(prepared, 'disabled_backend_key_completion_rejected',
    manifest, PATH, { expectBadAuth: true });
  manifest.evidence.backend_rejection = {
    reason: rejected.reason, hash: rejected.hash,
    type: 'RPC ingestion rejection, not a confirmed ledger transaction',
  };
  const after = await balances();
  if (JSON.stringify(after) !== JSON.stringify(manifest.evidence.balances_before_completion)) {
    throw new Error('Balances changed during rejected completion');
  }
  manifest.evidence.balances_after_rejection = after;
  const unsigned = TransactionBuilder.cloneFrom(prepared).build();
  if (unsigned.signatures.length !== 0) throw new Error('Unexpected signatures in approval request');
  await writeFile(resolve(OUT, 'complete-campaign-1.xdr.txt'), unsigned.toXDR() + '\n');
  manifest.approval = {
    status: 'requires_two_member_wallet_signatures', hash: unsigned.hash().toString('hex'),
    unsigned_xdr: unsigned.toXDR(), source: manifest.accounts.settlement,
    campaign_id: 1, amount_base_units: '10000000', expected_split: '82/10/5/2/1',
    signature_deadline_utc: new Date(Number(unsigned.timeBounds.maxTime) * 1000).toISOString(),
  };
  manifest.status = 'deployed_and_locked_to_2_of_3; quorum_completion_pending';
  await saveManifest(PATH, manifest);
  console.log('TESTNET_DEPLOYMENT_READY ' + PATH);
} catch (error) {
  manifest.status = 'incomplete; do not claim acceptance';
  manifest.failure = error.message;
  await saveManifest(PATH, manifest);
  throw error;
}