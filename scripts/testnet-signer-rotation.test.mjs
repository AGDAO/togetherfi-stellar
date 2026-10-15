import test from 'node:test';
import assert from 'node:assert/strict';
import { Account, Keypair, Networks, TransactionBuilder } from '@stellar/stellar-sdk';
import { buildSignerRotation, rotationConfiguration, ZYAD_TESTNET_KEY } from './prepare-testnet-signer-rotation.mjs';

function fixture() {
  const keys = Array.from({ length: 5 }, () => Keypair.random().publicKey());
  const manifest = {
    network: 'testnet',
    signers: { Ayoub: keys[0], Ace: keys[1], King: keys[2] },
    accounts: { governance: keys[3], settlement: keys[4] },
  };
  const account = key => Object.assign(new Account(key, '100'), {
    account_id: key,
    thresholds: { low_threshold: 2, med_threshold: 2, high_threshold: 2 },
    signers: [...keys.slice(0, 3).map(key => ({ key, weight: 1, type: 'ed25519_public_key' })),
      { key, weight: 0, type: 'ed25519_public_key' }],
  });
  return { manifest, governance: account(keys[3]), settlement: account(keys[4]) };
}

test('rotation only adds Zyad and removes Ace from both dedicated roles', () => {
  const { manifest, governance, settlement } = fixture();
  const { transaction, config } = buildSignerRotation(manifest, governance, settlement, ZYAD_TESTNET_KEY);
  assert.equal(transaction.source, manifest.accounts.governance);
  assert.equal(transaction.fee, '40000');
  assert.equal(transaction.signatures.length, 0);
  assert.equal(transaction.operations.length, 4);
  for (let i = 0; i < 4; i++) {
    const operation = transaction.operations[i];
    assert.equal(operation.type, 'setOptions');
    assert.equal(operation.source, i < 2 ? manifest.accounts.governance : manifest.accounts.settlement);
    assert.deepEqual(operation.signer, {
      ed25519PublicKey: i % 2 ? manifest.signers.Ace : ZYAD_TESTNET_KEY,
      weight: i % 2 ? 0 : 1,
    });
    for (const field of ['masterWeight', 'lowThreshold', 'medThreshold', 'highThreshold', 'setFlags', 'clearFlags']) {
      assert.equal(operation[field], undefined);
    }
  }
  assert.deepEqual(config.signers, { Ayoub: manifest.signers.Ayoub, Zyad: ZYAD_TESTNET_KEY, King: manifest.signers.King });
  assert.ok(manifest.signers.Ace, 'historical deployment evidence remains unchanged');
  const roundtrip = TransactionBuilder.fromXDR(transaction.toXDR(), Networks.TESTNET);
  assert.equal(roundtrip.hash().toString('hex'), transaction.hash().toString('hex'));
  assert.notEqual(TransactionBuilder.fromXDR(transaction.toXDR(), Networks.PUBLIC).hash().toString('hex'), transaction.hash().toString('hex'));
});

test('rotation rejects wrong network, missing or duplicate signers, and member-wallet role accounts', () => {
  const { manifest } = fixture();
  assert.throws(() => rotationConfiguration({ ...manifest, network: 'public' }, ZYAD_TESTNET_KEY));
  assert.throws(() => rotationConfiguration(manifest, 'not-a-public-key'));
  assert.throws(() => rotationConfiguration(manifest, manifest.signers.King));
  assert.throws(() => rotationConfiguration({ ...manifest, accounts: {
    ...manifest.accounts, governance: manifest.signers.Ayoub,
  } }, ZYAD_TESTNET_KEY));
  assert.throws(() => rotationConfiguration({ ...manifest, accounts: {
    ...manifest.accounts, settlement: manifest.accounts.governance,
  } }, ZYAD_TESTNET_KEY));
});

test('rotation fails closed if either account has unexpected quorum, active master, or identity', () => {
  const make = () => fixture();
  for (const role of ['governance', 'settlement']) {
    const f = make();
    f[role].thresholds.high_threshold = 1;
    assert.throws(() => buildSignerRotation(f.manifest, f.governance, f.settlement, ZYAD_TESTNET_KEY));
    const g = make();
    g[role].signers.at(-1).weight = 1;
    assert.throws(() => buildSignerRotation(g.manifest, g.governance, g.settlement, ZYAD_TESTNET_KEY));
    const h = make();
    h[role].account_id = Keypair.random().publicKey();
    assert.throws(() => buildSignerRotation(h.manifest, h.governance, h.settlement, ZYAD_TESTNET_KEY));
  }
  const f = make();
  assert.throws(() => buildSignerRotation(f.manifest, f.governance, f.settlement, ZYAD_TESTNET_KEY, 0));
  assert.throws(() => buildSignerRotation(f.manifest, f.governance, f.settlement, ZYAD_TESTNET_KEY, 86401));
});
