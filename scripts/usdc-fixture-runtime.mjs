import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { access, mkdir, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import {
  Address, Contract, Keypair, Operation, authorizeEntry, scValToNative,
} from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';
import {
  NETWORK, server, horizon, activate, basicTransaction, addressValue,
  saveManifest, submitPrepared,
} from './testnet-common.mjs';
import { STELLAR_USDC, testnetUsdcAsset } from './stellar-usdc.mjs';

export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
export const feeCap = 100_000_000n; // 10 free testnet XLM, not production policy.

export function expectedUsdcSplit(total) {
  if (typeof total !== 'bigint' || total <= 0n) throw new Error('Positive base-unit amount required');
  const parts = {
    service: total * 1000n / 10000n, nft_pool: total * 500n / 10000n,
    revenue_pool: total * 200n / 10000n, referrer: total * 100n / 10000n,
  };
  parts.creator = total - Object.values(parts).reduce((a, b) => a + b, 0n);
  return parts;
}

export async function createUsdcFixture(buildPath, outputPath) {
  const asset = testnetUsdcAsset(NETWORK);
  assert.equal((await server.getNetwork()).passphrase, NETWORK);
  const build = JSON.parse(await readFile(buildPath, 'utf8'));
  assert.equal(build.network_authorization, 'testnet-only');
  assert.equal(build.source_repository, 'AGDAO/togetherfi-stellar');
  assert.equal(build.initialization, 'deployer-preimage-v1');
  assert.equal(build.builder_commit, '4a554f0e75c245cf4d3ea5f98f3a653d51edcd18');
  assert.match(build.source_commit, /^[a-f0-9]{40}$/);
  const binaries = {};
  for (const name of ['campaign_escrow', 'contributor_pool']) {
    const spec = build.packages[name];
    assert.equal(spec.attestation_verified, true);
    binaries[name] = await readFile(spec.file);
    assert.equal(sha256(binaries[name]), build.wasm_sha256[name]);
    const verification = JSON.parse(await readFile(resolve(spec.file, '..', spec.verification_file), 'utf8'));
    assert.ok(Array.isArray(verification) && verification.length > 0,
      'Missing retained cryptographic verification result');
    // Reverify actual bytes: a caller-supplied boolean/JSON file is not proof.
    execFileSync('gh', [
      'attestation', 'verify', spec.file, '--repo', build.source_repository,
      '--signer-workflow', 'stellar-expert/soroban-build-workflow/.github/workflows/release.yml',
      '--signer-digest', build.builder_commit, '--source-digest', build.source_commit,
      '--deny-self-hosted-runners',
    ], {
      env: { ...process.env, ...(process.env.GITHUB_WORKFLOW_TOKEN
        ? { GH_TOKEN: process.env.GITHUB_WORKFLOW_TOKEN } : {}) },
      stdio: ['ignore', 'pipe', 'pipe'], timeout: 120_000,
    });
  }
  const path = resolve(outputPath);
  try { await access(path); throw new Error('Existing fixture evidence must not be overwritten'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  await mkdir(resolve(path, '..'), { recursive: true });
  const keys = Object.fromEntries(['deployer', 'governance', 'settlement', 'sponsor', 'creator', 'referrer']
    .map(name => [name, Keypair.random()]));
  const members = [Keypair.random(), Keypair.random(), Keypair.random()];
  let automatedQuorum = false;
  const accounts = Object.fromEntries(Object.entries(keys).map(([name, key]) => [name, key.publicKey()]));
  accounts.service_treasury = accounts.deployer;
  accounts.eligibility_authority = accounts.deployer;
  const manifest = {
    network: 'testnet', created_at: new Date().toISOString(),
    status: 'fresh_usdc_fixture_setup',
    source_commit: build.source_commit, builder_commit: build.builder_commit,
    wasm_sha256: build.wasm_sha256,
    source_review: 'pending; no independent approval claimed',
    asset: { ...STELLAR_USDC.testnet, contract: STELLAR_USDC.testnet.sac },
    accounts, contracts: {}, transactions: [], evidence: {},
    automation_members: members.map(key => key.publicKey()),
    automation_scope: 'Disposable test signers only; not signatures by Ayoub, Zyad or King.',
    designated_members: {
      Ayoub: 'GDGWL4MGDMHYLDKWTKOMH6RY2B4LR3A4GDUVGXS64LKPLSDTBL4PVBLE',
      Zyad: 'GDTAV2MA7TX4ATKBPEF4G7SFT7GHPIRZ4UVMQKNISPEGC5AE4P3AWWG7',
      King: 'GARRSD6FDAUASB5UMPM6KV4UQZGWO47NBUY6F2GFC4V4UE6YOWRZDS5U',
    },
    custody: 'Free testnet fixtures. Member seeds never read; fixture seeds live in process memory only.',
  };
  const save = () => saveManifest(path, manifest);
  await save();
  const invoke = (id, method, args = []) => new Contract(id).call(method, ...args);
  async function view(id, method, args = []) {
    const raw = await basicTransaction(accounts.deployer, [invoke(id, method, args)]);
    const simulated = await server.simulateTransaction(raw);
    if (!rpc.Api.isSimulationSuccess(simulated)) throw new Error(`${method} read failed: ${simulated.error}`);
    return scValToNative(simulated.result.retval);
  }
  function signingKeys(source) {
    if (automatedQuorum && [accounts.governance, accounts.settlement].includes(source)) return members.slice(0, 2);
    const key = Object.values(keys).find(k => k.publicKey() === source);
    if (!key) throw new Error('No permitted fixture source');
    return [key];
  }
  async function prepare(source, operation, timeout = 600, maximumFee = feeCap) {
    const raw = await basicTransaction(source, [operation], timeout);
    const simulated = await server.simulateTransaction(raw);
    if (!rpc.Api.isSimulationSuccess(simulated)) throw new Error('Simulation failed: ' + simulated.error);
    raw.operations[0].auth = await Promise.all(simulated.result.auth.map(async entry => {
      if (entry.credentials().switch().name === 'sorobanCredentialsSourceAccount') return entry;
      const address = Address.fromScAddress(entry.credentials().address().address()).toString();
      const key = Object.values(keys).find(k => k.publicKey() === address);
      if (!key || [accounts.governance, accounts.settlement].includes(address)) {
        throw new Error('Unexpected additional fixture authorization');
      }
      return authorizeEntry(entry, key, simulated.latestLedger + 200, NETWORK);
    }));
    const prepared = rpc.assembleTransaction(raw, simulated).build();
    assert.ok(maximumFee > 0n && maximumFee <= 200_000_000n);
    if (BigInt(prepared.fee) > maximumFee) {
      throw new Error(`Free-test-XLM quote ${prepared.fee} stroops exceeds cap ${maximumFee}`);
    }
    return prepared;
  }
  async function send(source, operation, label, maximumFee = feeCap) {
    const prepared = await prepare(source, operation, 600, maximumFee);
    prepared.sign(...signingKeys(source));
    return submitPrepared(prepared, label, manifest, path);
  }
  async function call(source, id, method, args, label) {
    return send(source, invoke(id, method, args), label);
  }
  async function reject(source, id, method, args, code, label) {
    const simulated = await server.simulateTransaction(await basicTransaction(source, [invoke(id, method, args)]));
    assert.ok(!rpc.Api.isSimulationSuccess(simulated), 'Unexpectedly allowed: ' + label);
    assert.match(simulated.error || '', new RegExp(`Error\\(Contract, #${code}\\)`),
      'Only the intended contract rejection counts, not RPC/auth/timeout failure');
    manifest.evidence[label] = {
      result: 'expected_contract_simulation_rejection', expected_error_code: code,
      error: simulated.error, confirmed_transaction: false,
    };
    await save();
  }
  async function usdcBalance(id) {
    return BigInt(await view(manifest.asset.contract, 'balance', [addressValue(id)]));
  }
  async function balances() {
    const destinations = {
      creator: accounts.creator, service: accounts.service_treasury,
      nft_pool: manifest.contracts.nft_pool.id, revenue_pool: manifest.contracts.revenue_pool.id,
      referrer: accounts.referrer, escrow: manifest.contracts.escrow.id,
    };
    return Object.fromEntries(await Promise.all(Object.entries(destinations)
      .map(async ([name, id]) => [name, String(await usdcBalance(id))])));
  }
  async function activateAccountsAndTrustlines() {
    for (const key of Object.values(keys)) await activate(key.publicKey());
    const trust = await basicTransaction(accounts.deployer, Object.values(keys).map(key =>
      Operation.changeTrust({ asset, limit: '20', source: key.publicKey() })));
    trust.sign(...Object.values(keys));
    await submitPrepared(trust, 'circle_usdc_fixture_trustlines', manifest, path);
    assert.equal(Number(await view(manifest.asset.contract, 'decimals')), 7);
    assert.equal(await view(manifest.asset.contract, 'symbol'), 'USDC');
  }
  return {
    build, binaries, manifest, accounts, keys, members, path, save, invoke, view,
    prepare, send, call, reject, balances, usdcBalance, activateAccountsAndTrustlines,
    signingKeys, setAutomatedQuorum() { automatedQuorum = true; },
  };
}
