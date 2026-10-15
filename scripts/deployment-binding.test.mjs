import assert from 'node:assert/strict';
import test from 'node:test';
import { Keypair, scValToNative } from '@stellar/stellar-sdk';
import { contractId } from './testnet-common.mjs';
import { deploymentProof, verifyDeploymentBindings } from './deployment-binding.mjs';

test('deployment proof uses the original address and exact salt', () => {
  const deployer = Keypair.random().publicKey();
  const salt = Buffer.alloc(32, 7);
  const proof = deploymentProof(contractId(deployer, salt), deployer, salt);
  assert.equal(scValToNative(proof[0]), deployer);
  assert.deepEqual(Buffer.from(scValToNative(proof[1])), salt);
  assert.throws(() => deploymentProof(contractId(deployer, salt), Keypair.random().publicKey(), salt));
  assert.throws(() => deploymentProof(contractId(deployer, salt), deployer, Buffer.alloc(32, 8)));
  assert.throws(() => deploymentProof(contractId(deployer, salt), deployer, Buffer.alloc(31)));
});

const manifest = {
  contracts: { escrow: { id: 'escrow' }, nft_pool: { id: 'nft' }, revenue_pool: { id: 'revenue' } },
  accounts: { governance: 'governance', settlement: 'settlement', service_treasury: 'treasury', eligibility_authority: 'eligibility' },
  asset: { contract: 'token' },
};
const configured = (id, method) => ({
  governance: 'governance', settlement_authority: 'settlement', token_contract: 'token',
  service_treasury: 'treasury', pool_destinations: ['nft', 'revenue'],
  eligibility_authority: 'eligibility', escrow_contract: 'escrow',
})[method];

test('deployment verifies all thirteen authority, token and destination bindings', async () => {
  assert.deepEqual(await verifyDeploymentBindings(manifest, configured), { verified: true, checks: 13 });
});

test('every individual mismatched binding blocks deployment', async () => {
  const seen = [];
  await verifyDeploymentBindings(manifest, (id, method) => {
    seen.push([id, method]);
    return configured(id, method);
  });
  for (const [target, getter] of seen) {
    await assert.rejects(verifyDeploymentBindings(manifest, (id, method) =>
      id === target && method === getter ? 'wrong' : configured(id, method)), /binding mismatch/);
  }
});

test('failed configuration reads fail closed', async () => {
  await assert.rejects(verifyDeploymentBindings(manifest, () => { throw new Error('RPC unavailable'); }), /RPC unavailable/);
});
