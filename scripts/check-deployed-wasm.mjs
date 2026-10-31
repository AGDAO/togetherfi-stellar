// Read-only: retrieve deployed WASM via public RPC and compare recorded hashes.
// Never prepares, signs, submits, restores or redeploys a transaction.
import * as rpc from '@stellar/stellar-sdk/rpc';
import { createHash } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname } from 'node:path';

const output = process.argv[2];
if (!output) throw new Error('Usage: node check-deployed-wasm.mjs <output.json>');
const server = new rpc.Server('https://soroban-testnet.stellar.org', { timeout: 20_000 });
const legacyEscrow = '1cfbad51dc1932e81de857b4a22402f883584ba4c6b89388aff53929c2e6b0eb';
const legacyPool = '2f24a684373cfb2683912b1a85be0b8e4ac859fb64c15b14a52050579e99a865';
const usdcEscrow = '6f1a3e10abd79b76cbb4101f5ea42da5e51ad0d45950093e0cacf602fd51b0ff';
const usdcPool = '0842b07ff7056fcfce1df0e0586bf1779de57d4c6e4c89014a0af220d976c14f';
const instances = [
  ['historical-xlm-escrow', 'CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ', legacyEscrow],
  ['historical-xlm-nft-pool', 'CAUPWNNQLQLURIJJTFW7LURJRD2OLGV4R5E6LNSUJXKSESRVGO54YP76', legacyPool],
  ['historical-xlm-revenue-pool', 'CD6TNSBSLAPXE4JXDBNTNEMMXH7DAWGUA3TR5HMP4QQIHCYM7544ED25', legacyPool],
  ['attested-usdc-escrow', 'CCUYLTWIPHUMXGJNPQZZELDGJFQWXMLFW3JW3VAAYRNAMTVLADUWZS2U', usdcEscrow],
  ['attested-usdc-nft-pool', 'CDSHQH4FICZ5RFJG677UP4XTECOAO6IIJTQWB7KN2PQJXGOENEJ6BENT', usdcPool],
  ['attested-usdc-revenue-pool', 'CACX36IJNDCAZKXEP2DC2ZDJK4QF4R7BHFFXJTSZKJ77K74OP6HAGRQR', usdcPool],
];
const results = await Promise.all(instances.map(async ([name, contract, expected]) => {
  try {
    const bytes = await server.getContractWasmByContractId(contract);
    const actual = createHash('sha256').update(bytes).digest('hex');
    return { name, contract, expected, actual, bytes: bytes.length, match: actual === expected };
  } catch (error) {
    return { name, contract, expected, match: false, error: error.message };
  }
}));
await mkdir(dirname(output), { recursive: true });
await writeFile(output, JSON.stringify({
  checkedAt: new Date().toISOString(),
  network: 'testnet',
  rpc: 'https://soroban-testnet.stellar.org',
  readOnly: true,
  results,
}, null, 2) + '\n');
for (const result of results) {
  console.log(result.name, result.match ? 'MATCH' : 'NOT_CONFIRMED',
    result.actual || result.error);
}
if (results.some(result => !result.match)) {
  throw new Error('Not all deployed hashes confirmed; inspect the saved evidence');
}
