// Prepares public, unsigned testnet data only. Never signs or broadcasts.
import { readFile, writeFile, mkdir, access } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Operation, TransactionBuilder } from '@stellar/stellar-sdk';
import { NETWORK, horizon, validateSigners, assertQuorum, json } from './testnet-common.mjs';

export const ZYAD_TESTNET_KEY = 'GDTAV2MA7TX4ATKBPEF4G7SFT7GHPIRZ4UVMQKNISPEGC5AE4P3AWWG7';

export function rotationConfiguration(manifest, replacement) {
  const previous = validateSigners(manifest);
  if (!manifest.signers.Ace || manifest.signers.Zyad) {
    throw new Error('Expected the existing Ace signer; do not overwrite completed evidence');
  }
  const signers = { Ayoub: manifest.signers.Ayoub, Zyad: replacement, King: manifest.signers.King };
  const next = validateSigners({ network: manifest.network, signers });
  const roles = [manifest.accounts.governance, manifest.accounts.settlement];
  if (new Set(roles).size !== 2 || roles.some(role => previous.includes(role) || next.includes(role))) {
    throw new Error('Governance and settlement must remain distinct dedicated role accounts');
  }
  if (previous.includes(replacement)) throw new Error('Replacement must be a new signer');
  return { previous, signers, next, roles };
}

export function buildSignerRotation(manifest, governance, settlement, replacement, timeout = 86400) {
  const config = rotationConfiguration(manifest, replacement);
  if (governance.account_id !== config.roles[0] || settlement.account_id !== config.roles[1]) {
    throw new Error('Account data does not match the deployed role accounts');
  }
  assertQuorum(governance, config.previous);
  assertQuorum(settlement, config.previous);
  if (!Number.isInteger(timeout) || timeout < 60 || timeout > 86400) {
    throw new Error('Signing window must be between one minute and one day');
  }
  const builder = new TransactionBuilder(governance, {
    fee: '10000', networkPassphrase: NETWORK,
  });
  for (const source of config.roles) {
    builder.addOperation(Operation.setOptions({
      source, signer: { ed25519PublicKey: replacement, weight: 1 },
    }));
    builder.addOperation(Operation.setOptions({
      source, signer: { ed25519PublicKey: manifest.signers.Ace, weight: 0 },
    }));
  }
  // No threshold/master changes, payments, contract calls, or member-wallet operations.
  const transaction = builder.setTimeout(timeout).build();
  if (transaction.signatures.length) throw new Error('Preparation must remain unsigned');
  return { transaction, config };
}

async function main() {
  if (process.argv[2] !== '--prepare-testnet') {
    throw new Error('Explicit --prepare-testnet required; mainnet is unsupported');
  }
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  const out = resolve(root, 'evidence/testnet-signer-rotation');
  const requestPath = resolve(out, 'request.json');
  try {
    await access(requestPath);
    throw new Error('An existing signing request must not be overwritten or silently refreshed');
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  const manifest = JSON.parse(await readFile(resolve(root, 'evidence/testnet-escrow-v2/manifest.json'), 'utf8'));
  // Validate before making network calls. Public account reads only.
  rotationConfiguration(manifest, ZYAD_TESTNET_KEY);
  const [governance, settlement] = await Promise.all([
    horizon.loadAccount(manifest.accounts.governance),
    horizon.loadAccount(manifest.accounts.settlement),
  ]);
  const { transaction, config } = buildSignerRotation(manifest, governance, settlement, ZYAD_TESTNET_KEY);
  const request = {
    network: 'testnet',
    status: 'awaiting_existing_quorum_signatures',
    prepared_at: new Date().toISOString(),
    expires_at: new Date(Number(transaction.timeBounds.maxTime) * 1000).toISOString(),
    accounts: { governance: config.roles[0], settlement: config.roles[1] },
    before_signers: manifest.signers,
    after_signers: config.signers,
    thresholds_unchanged: { low: 2, medium: 2, high: 2, master_weight: 0 },
    preferred_authorizers: ['Ayoub', 'King'],
    fee_test_xlm: Number(transaction.fee) / 10000000,
    hash: transaction.hash().toString('hex'),
    unsigned_xdr: transaction.toXDR(),
    observed_accounts: [governance, settlement].map(account => ({
      account_id: account.account_id, sequence: account.sequence,
      thresholds: account.thresholds, signers: account.signers,
    })),
    note: 'Unsigned request only. Existing deployed manifest and on-chain signer set are unchanged.',
  };
  await mkdir(out, { recursive: true });
  await writeFile(resolve(out, 'replace-ace-with-zyad.xdr.txt'), transaction.toXDR() + '\n', { flag: 'wx' });
  await writeFile(requestPath, json(request), { flag: 'wx' });
  console.log(json({
    status: request.status, hash: request.hash, expires_at: request.expires_at,
    fee_test_xlm: request.fee_test_xlm, operations: transaction.operations.length,
    request: requestPath, broadcast: false,
  }));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
