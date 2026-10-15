import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { freeTestFeeCap, validateRotatedEvidence } from './prepare-rotated-testnet-approval.mjs';

const root = new URL('../evidence/', import.meta.url);
const load = file => JSON.parse(readFileSync(new URL(file, root), 'utf8'));
const fixture = () => ({
  baseline: load('testnet-escrow-v2/manifest.json'),
  rotation: load('testnet-signer-rotation/confirmation.json'),
});

test('free-testnet resource headroom is quote-based and bounded below uint32 overflow', () => {
  assert.equal(freeTestFeeCap('353653342'), 708306684n);
  assert.throws(() => freeTestFeeCap('-1'));
  assert.throws(() => freeTestFeeCap('not-a-fee'));
  assert.throws(() => freeTestFeeCap('1000000000'));
});

test('new-roster fixture uses completed native testnet baseline and both confirmed role accounts', () => {
  const { baseline, rotation } = fixture();
  assert.deepEqual(validateRotatedEvidence(baseline, rotation),
    Object.values(rotation.after_signers));
  assert.ok(baseline.signers.Ace, 'Historical roster must not be rewritten');
  assert.equal(baseline.approval.status, 'confirmed');
});

test('new-roster fixture rejects wrong network, pending evidence, wrong members and broken quorum', () => {
  for (const mutate of [
    (b, r) => { b.network = 'public'; },
    (b, r) => { r.network = 'public'; },
    (b, r) => { r.status = 'awaiting_signatures'; },
    (b, r) => { r.after_signers.Zyad = b.signers.Ace; },
    (b, r) => { r.accounts[0].thresholds.high_threshold = 1; },
    (b, r) => { r.accounts[1].signers.at(-1).weight = 1; },
    (b, r) => { b.accounts.settlement = b.accounts.governance; },
    (b, r) => { b.approval.status = 'pending'; },
    (b, r) => { b.asset.contract = 'not-the-native-testnet-asset'; },
  ]) {
    const { baseline, rotation } = fixture();
    mutate(baseline, rotation);
    assert.throws(() => validateRotatedEvidence(baseline, rotation));
  }
});
