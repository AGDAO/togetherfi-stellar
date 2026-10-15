import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { Keypair, Networks, TransactionBuilder, xdr } from '@stellar/stellar-sdk';
import { freeTestFeeCap, releaseFixtureSpec, validateRotatedEvidence } from './prepare-rotated-testnet-approval.mjs';

const root = new URL('../evidence/', import.meta.url);
const load = file => JSON.parse(readFileSync(new URL(file, root), 'utf8'));
const fixture = () => ({
  baseline: load('testnet-escrow-v2/manifest.json'),
  rotation: load('testnet-signer-rotation/confirmation.json'),
});

test('additional signer pair uses its own campaign without replacing prior evidence', () => {
  assert.deepEqual(releaseFixtureSpec(), {
    pair: 'zyad-king', campaignId: 3, signers: ['Zyad', 'King'],
    directory: 'testnet-zyad-king',
  });
  assert.deepEqual(releaseFixtureSpec('zyad-ayoub'), {
    pair: 'zyad-ayoub', campaignId: 4, signers: ['Zyad', 'Ayoub'],
    directory: 'testnet-zyad-ayoub',
  });
  assert.throws(() => releaseFixtureSpec('../mainnet'));
  assert.throws(() => releaseFixtureSpec('ace-king'));
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

for (const { directory, names, singleMember } of [
  { directory: 'testnet-zyad-king', names: ['Zyad', 'King'], singleMember: 'Zyad' },
  { directory: 'testnet-zyad-ayoub', names: ['Zyad', 'Ayoub'], singleMember: 'Ayoub' },
]) {
test(`recorded ${names.join(' and ')} release: valid signatures, expected block, exact payouts and liability`, () => {
  const m = load(`${directory}/manifest.json`);
  assert.equal(m.network, 'testnet');
  assert.equal(m.approval.status, 'confirmed');
  const signed = TransactionBuilder.fromXDR(
    readFileSync(new URL(`${directory}/quorum-signed.xdr.txt`, root), 'utf8').trim(),
    Networks.TESTNET);
  assert.equal(signed.hash().toString('hex'), m.approval.hash);
  assert.equal(signed.signatures.length, 2);
  for (const [index, name] of names.entries()) {
    const key = Keypair.fromPublicKey(m.signers[name]);
    assert.deepEqual(signed.signatures[index].hint(), key.signatureHint());
    assert.ok(key.verify(signed.hash(), signed.signatures[index].signature()));
  }
  assert.equal(m.evidence.one_of_three_rejection.signed_by, singleMember);
  assert.equal(m.evidence.one_of_three_rejection.reason, 'txBadAuth');
  const rejected = m.transactions.find(entry =>
    entry.label === `one_of_three_${singleMember.toLowerCase()}_completion_rejected`);
  assert.equal(rejected.status, 'rejected_before_ledger');
  assert.equal(xdr.TransactionResult.fromXDR(rejected.result_xdr, 'base64')
    .result().switch().name, 'txBadAuth');
  const single = TransactionBuilder.fromXDR(rejected.signed_xdr, Networks.TESTNET);
  assert.equal(single.hash().toString('hex'), m.approval.hash);
  assert.equal(single.signatures.length, 1);
  assert.ok(Keypair.fromPublicKey(m.signers[singleMember])
    .verify(single.hash(), single.signatures[0].signature()));
  assert.equal(m.evidence.ledger_receipt.successful, true);
  assert.equal(m.evidence.ledger_receipt.hash, m.approval.hash);
  assert.equal(m.evidence.campaign_after_completion.status[0], 'Completed');
  assert.deepEqual(m.evidence.balances_after_one_signature_rejection,
    m.evidence.balances_before_completion);
  const expected = { creator: 8_200_000n, service: 1_000_000n, nft_pool: 500_000n,
    revenue_pool: 200_000n, referrer: 100_000n, escrow: -10_000_000n };
  for (const [name, amount] of Object.entries(expected)) {
    assert.equal(BigInt(m.evidence.balances_after_completion[name]) -
      BigInt(m.evidence.balances_before_completion[name]), amount);
  }
  const before = m.evidence.accounting_before_completion;
  const after = m.evidence.accounting_after_completion;
  assert.equal(BigInt(before.escrow.TotalLiability) - BigInt(after.escrow.TotalLiability),
    10_000_000n);
  assert.equal(BigInt(after.nft_pool.TotalCredited) - BigInt(before.nft_pool.TotalCredited),
    500_000n);
  assert.equal(BigInt(after.revenue_pool.TotalCredited) -
    BigInt(before.revenue_pool.TotalCredited), 200_000n);
  assert.equal(m.evidence.historical_expiry_campaign_unchanged, true);
});
}

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
