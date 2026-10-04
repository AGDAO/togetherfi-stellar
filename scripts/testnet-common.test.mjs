import test from 'node:test';
import assert from 'node:assert/strict';
import { Keypair, Account, Networks, TransactionBuilder, Operation } from '@stellar/stellar-sdk';
import { validateSigners, assertQuorum, approvedSignatureCount, contractId } from './testnet-common.mjs';

const fixtures = Array.from({ length: 4 }, () => Keypair.random());
const keys = fixtures.slice(0, 3).map(key => key.publicKey());
const config = { network: 'testnet', signers: Object.fromEntries(keys.map((key, i) => [i, key])) };
const account = () => ({
  account_id: fixtures[3].publicKey(),
  thresholds: { low_threshold: 2, med_threshold: 2, high_threshold: 2 },
  signers: keys.map(key => ({ key, weight: 1, type: 'ed25519_public_key' })),
});

test('three distinct testnet signer keys accepted', () => assert.deepEqual(validateSigners(config), keys));
test('mainnet is prohibited', () => assert.throws(() => validateSigners({ ...config, network: 'mainnet' })));
test('missing, duplicate, and invalid keys rejected', () => {
  for (const signers of [{ a: keys[0] }, { a: keys[0], b: keys[0], c: keys[1] },
    { a: keys[0], b: keys[1], c: '0x123' }]) {
    assert.throws(() => validateSigners({ network: 'testnet', signers }));
  }
});
test('exact 2-of-3 account accepted', () => assert.doesNotThrow(() => assertQuorum(account(), keys)));
test('enabled master, extra signer, and weight-2 signer rejected', () => {
  for (const signer of [
    { key: fixtures[3].publicKey(), weight: 1, type: 'ed25519_public_key' },
    { key: keys[0], weight: 2, type: 'ed25519_public_key' },
  ]) {
    assert.throws(() => assertQuorum({ ...account(), signers: [...account().signers, signer] }, keys));
  }
});
test('all account thresholds must be 2', () => {
  for (const key of ['low_threshold', 'med_threshold', 'high_threshold']) {
    assert.throws(() => assertQuorum({
      ...account(), thresholds: { ...account().thresholds, [key]: 1 },
    }, keys));
  }
});
test('duplicate signatures do not constitute quorum', () => {
  const tx = new TransactionBuilder(new Account(fixtures[3].publicKey(), '0'), {
    fee: '100', networkPassphrase: Networks.TESTNET,
  }).addOperation(Operation.manageData({ name: 'fixture', value: 'test' })).setTimeout(60).build();
  tx.sign(fixtures[0], fixtures[0]);
  assert.equal(approvedSignatureCount(tx, keys), 1);
  tx.sign(fixtures[1]);
  assert.equal(approvedSignatureCount(tx, keys), 2);
});
test('contract addresses are deterministic and salt-specific', () => {
  const first = contractId(keys[0], Buffer.alloc(32, 1));
  assert.equal(first, contractId(keys[0], Buffer.alloc(32, 1)));
  assert.notEqual(first, contractId(keys[0], Buffer.alloc(32, 2)));
});