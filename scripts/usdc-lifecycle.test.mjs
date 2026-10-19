import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { expectedUsdcSplit } from './usdc-fixture-runtime.mjs';

test('one seven-decimal USDC reconciles to 82/10/5/2/1 without loss', () => {
  const parts = expectedUsdcSplit(10_000_000n);
  assert.deepEqual(parts, {
    creator: 8_200_000n, service: 1_000_000n,
    nft_pool: 500_000n, revenue_pool: 200_000n, referrer: 100_000n,
  });
});
test('integer remainder goes to creator and all base units reconcile', () => {
  for (const amount of [1n, 99n, 100n, 101n, 999_999n, 12_345_678_901n]) {
    const parts = expectedUsdcSplit(amount);
    assert.equal(Object.values(parts).reduce((a, b) => a + b, 0n), amount);
    assert.ok(Object.values(parts).every(n => n >= 0n));
  }
  assert.equal(expectedUsdcSplit(101n).creator, 83n);
  assert.throws(() => expectedUsdcSplit(0n));
  assert.throws(() => expectedUsdcSplit(-1n));
});
test('fresh runner cannot overwrite old evidence or imply human fixture signatures', () => {
  const runner = readFileSync(new URL('./run-fresh-usdc-lifecycle.mjs', import.meta.url), 'utf8');
  const runtime = readFileSync(new URL('./usdc-fixture-runtime.mjs', import.meta.url), 'utf8');
  assert.match(runner, /evidence\/testnet-usdc-escrow/);
  assert.doesNotMatch(runner, /evidence\/testnet-escrow-v2/);
  assert.match(runner, /requires_two_designated_member_wallet_signatures/);
  assert.match(runtime, /--source-digest/);
  assert.match(runtime, /--signer-digest/);
  assert.match(runtime, /server\.getNetwork/);
  assert.match(runtime, /No permitted fixture source/);
  assert.match(runtime, /Existing fixture evidence must not be overwritten/);
});
