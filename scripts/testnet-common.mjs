import { readFile, writeFile, rename, mkdir } from 'node:fs/promises';
import { dirname } from 'node:path';
import {
  Address, Horizon, Keypair, Networks, StrKey, TransactionBuilder, hash, xdr,
} from '@stellar/stellar-sdk';
import * as rpc from '@stellar/stellar-sdk/rpc';

export const NETWORK = Networks.TESTNET;
export const RPC_URL = 'https://soroban-testnet.stellar.org';
export const HORIZON_URL = 'https://horizon-testnet.stellar.org';
export const server = new rpc.Server(RPC_URL);
export const horizon = new Horizon.Server(HORIZON_URL);
export const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
export const addressValue = value => new Address(value).toScVal();
export const json = value => JSON.stringify(value, (_key, v) =>
  typeof v === 'bigint' ? v.toString() : v, 2) + '\n';

export function validateSigners(config) {
  if (config.network !== 'testnet') throw new Error('Only testnet is permitted');
  const keys = Object.values(config.signers ?? {});
  if (keys.length !== 3 || new Set(keys).size !== 3 ||
      !keys.every(key => StrKey.isValidEd25519PublicKey(key))) {
    throw new Error('Exactly three distinct Stellar public signer addresses required');
  }
  return keys;
}

export function assertQuorum(account, keys) {
  const active = account.signers.filter(signer => signer.weight > 0);
  if (active.length !== 3 || active.some(signer =>
    signer.type !== 'ed25519_public_key' || signer.weight !== 1 ||
    !keys.includes(signer.key))) throw new Error('Unexpected active signer configuration');
  if (keys.includes(account.account_id) ||
      active.some(signer => signer.key === account.account_id)) {
    throw new Error('Dedicated role account master key must be disabled');
  }
  if (Object.values(account.thresholds).some(value => value !== 2)) {
    throw new Error('All role-account thresholds must be 2');
  }
}

export function approvedSignatureCount(transaction, keys) {
  const digest = transaction.hash();
  return keys.filter(key => transaction.signatures.some(signature =>
    Keypair.fromPublicKey(key).verify(digest, signature.signature()))).length;
}

export function contractId(deployer, salt) {
  const preimage = xdr.HashIdPreimage.envelopeTypeContractId(
    new xdr.HashIdPreimageContractId({
      networkId: hash(Buffer.from(NETWORK)),
      contractIdPreimage: xdr.ContractIdPreimage.contractIdPreimageFromAddress(
        new xdr.ContractIdPreimageFromAddress({
          address: new Address(deployer).toScAddress(), salt,
        })),
    }));
  return StrKey.encodeContract(hash(preimage.toXDR()));
}

export async function saveManifest(path, manifest) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path + '.tmp', json(manifest), { mode: 0o600 });
  await rename(path + '.tmp', path);
}

export async function activate(publicKey) {
  try { return await horizon.loadAccount(publicKey); }
  catch (error) {
    if (error.response?.status !== 404) throw error;
  }
  const response = await fetch('https://friendbot.stellar.org/?addr=' + publicKey,
    { signal: AbortSignal.timeout(45000) });
  if (!response.ok) throw new Error('Testnet Friendbot failed: ' + response.status);
  await response.json();
  return horizon.loadAccount(publicKey);
}

export async function waitConfirmed(txHash) {
  for (let attempt = 0; attempt < 90; attempt++) {
    const result = await server.getTransaction(txHash);
    if (result.status === 'SUCCESS') return result;
    if (result.status === 'FAILED') throw new Error('Confirmed transaction failed: ' + txHash);
    await sleep(1500);
  }
  throw new Error('Transaction confirmation pending: ' + txHash);
}

export async function submitPrepared(transaction, label, manifest, path,
  { expectBadAuth = false } = {}) {
  const entry = {
    label, hash: transaction.hash().toString('hex'),
    signed_xdr: transaction.toXDR(), status: 'signed_not_broadcast',
  };
  manifest.transactions.push(entry);
  // These are public signed envelopes, not seeds. Persist before broadcasting.
  await saveManifest(path, manifest);
  let sent;
  try { sent = await server.sendTransaction(transaction); }
  catch (error) {
    entry.status = 'broadcast_outcome_unknown';
    await saveManifest(path, manifest);
    if (expectBadAuth) throw error;
    sent = { status: 'UNKNOWN' };
  }
  if (sent.status === 'ERROR') {
    entry.status = 'rejected_before_ledger';
    entry.result_xdr = sent.errorResult?.toXDR('base64');
    const reason = sent.errorResult?.result().switch().name;
    entry.reason = reason;
    await saveManifest(path, manifest);
    if (expectBadAuth && reason === 'txBadAuth') return entry;
    throw new Error(label + ' rejected: ' + reason);
  }
  if (expectBadAuth) throw new Error('Disabled setup key unexpectedly accepted');
  const result = await waitConfirmed(entry.hash);
  Object.assign(entry, {
    status: 'confirmed_success', ledger: result.ledger,
    ledger_closed_at: result.createdAt,
    explorer: 'https://stellar.expert/explorer/testnet/tx/' + entry.hash,
  });
  await saveManifest(path, manifest);
  console.log(label + ': ' + entry.hash);
  return result;
}

export async function loadJson(path) {
  return JSON.parse(await readFile(path, 'utf8'));
}

export async function basicTransaction(publicKey, operations, timeout = 600) {
  const account = await server.getAccount(publicKey);
  const builder = new TransactionBuilder(account, {
    fee: '10000', networkPassphrase: NETWORK,
  });
  for (const operation of operations) builder.addOperation(operation);
  return builder.setTimeout(timeout).build();
}