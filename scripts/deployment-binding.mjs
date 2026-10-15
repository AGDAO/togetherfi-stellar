import assert from 'node:assert/strict';
import { nativeToScVal } from '@stellar/stellar-sdk';
import { addressValue, contractId } from './testnet-common.mjs';

// Public deployment data is not a secret. Security comes from requiring the
// matching deployer's authorization inside initialize, after checking this ID.
export function deploymentProof(id, deployer, salt) {
  if (!Buffer.isBuffer(salt) || salt.length !== 32) throw new Error('Deployment salt must be 32 bytes');
  assert.equal(contractId(deployer, salt), id, 'Contract ID does not match deployer and salt');
  return [addressValue(deployer), nativeToScVal(salt)];
}

export async function verifyDeploymentBindings(manifest, view) {
  const escrow = manifest.contracts.escrow.id;
  const token = manifest.asset.contract;
  const checks = [
    [escrow, 'governance', manifest.accounts.governance],
    [escrow, 'settlement_authority', manifest.accounts.settlement],
    [escrow, 'token_contract', token],
    [escrow, 'service_treasury', manifest.accounts.service_treasury],
    [escrow, 'pool_destinations', [manifest.contracts.nft_pool.id, manifest.contracts.revenue_pool.id]],
  ];
  for (const name of ['nft_pool', 'revenue_pool']) {
    checks.push(
      [manifest.contracts[name].id, 'governance', manifest.accounts.governance],
      [manifest.contracts[name].id, 'eligibility_authority', manifest.accounts.eligibility_authority],
      [manifest.contracts[name].id, 'token_contract', token],
      [manifest.contracts[name].id, 'escrow_contract', escrow],
    );
  }
  for (const [id, method, expected] of checks) {
    assert.deepEqual(await view(id, method), expected, `Deployment binding mismatch: ${id}.${method}`);
  }
  return { verified: true, checks: checks.length };
}
