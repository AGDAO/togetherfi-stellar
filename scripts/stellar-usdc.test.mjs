import assert from 'node:assert/strict';
import test from 'node:test';
import { Asset, Networks } from '@stellar/stellar-sdk';
import {
  STELLAR_USDC, testnetUsdcAsset, assertUsdcTransferDeltas,
} from './stellar-usdc.mjs';

test('Circle asset identities derive the pinned SAC addresses on each network', () => {
  for (const [name, passphrase] of [
    ['testnet', Networks.TESTNET], ['mainnet', Networks.PUBLIC],
  ]) {
    const config = STELLAR_USDC[name];
    assert.equal(new Asset(config.code, config.issuer).contractId(passphrase), config.sac);
    assert.equal(config.decimals, 7);
  }
  assert.equal(testnetUsdcAsset(Networks.TESTNET).issuer, STELLAR_USDC.testnet.issuer);
  assert.throws(() => testnetUsdcAsset(Networks.PUBLIC), /Testnet only/);
  assert.throws(() => testnetUsdcAsset('custom'), /Testnet only/);
});

test('USDC receipt requires exact debit and credit in seven-decimal base units', () => {
  const before = { sender: 200_000_000n, recipient: 0n };
  assertUsdcTransferDeltas(before, { sender: 199_900_000n, recipient: 100_000n }, 100_000n);
  assert.throws(() =>
    assertUsdcTransferDeltas(before, { sender: 199_900_000n, recipient: 10_000n }, 100_000n));
  assert.throws(() => assertUsdcTransferDeltas(before, before, 0n));
});
