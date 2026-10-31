// StellarExpert accepts registration from real GitHub Actions runners only.
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

const [input, runId] = process.argv.slice(2);
if (process.env.GITHUB_ACTIONS !== 'true' || !input || !/^[0-9]+$/.test(runId || '')) {
  throw new Error('Run only in GitHub Actions with an attested manifest and build run ID');
}
const build = JSON.parse(await readFile(input, 'utf8'));
if (build.source_repository !== 'AGDAO/togetherfi-stellar' ||
    !/^[a-f0-9]{40}$/.test(build.source_commit)) throw new Error('Unexpected source identity');
const responses = [];
for (const [name, contractHash] of Object.entries(build.wasm_sha256)) {
  if (!/^[a-f0-9]{64}$/.test(contractHash) || !build.packages[name]?.attestation_verified) {
    throw new Error('Unverified build input');
  }
  const request = {
    repository: 'https://github.com/AGDAO/togetherfi-stellar',
    commitHash: build.source_commit, jobId: 'build', runId,
    contractHash, relativePath: 'contracts/' + name,
    packageName: name.replaceAll('_', '-'), makeTarget: 'ci-prepare',
  };
  const response = await fetch(
    'https://api.stellar.expert/explorer/public/contract-validation/match',
    {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request), signal: AbortSignal.timeout(45_000),
    });
  const body = await response.text();
  responses.push({ name, status: response.status, request, response: body });
  await writeFile(resolve(dirname(input), 'explorer-registration.json'),
    JSON.stringify(responses, null, 2) + '\n');
  console.log(name, response.status, body);
  if (!response.ok) throw new Error('Explorer registration failed; do not claim verified source');
}
console.log('REGISTRATION_ACCEPTED: verify explorer matching status separately.');

// Read the actual deployed instances from the same permitted runner.
// Registration acknowledgement must never be mistaken for source verification.
const instances = [
  ['historical-xlm-escrow', 'CC7EVM46T45WMEXMDKGCVTJ6RHVTDG24WZU3WQEPU5IW7EBOJJD3LPOZ'],
  ['historical-xlm-nft-pool', 'CAUPWNNQLQLURIJJTFW7LURJRD2OLGV4R5E6LNSUJXKSESRVGO54YP76'],
  ['historical-xlm-revenue-pool', 'CD6TNSBSLAPXE4JXDBNTNEMMXH7DAWGUA3TR5HMP4QQIHCYM7544ED25'],
  ['attested-usdc-escrow', 'CCUYLTWIPHUMXGJNPQZZELDGJFQWXMLFW3JW3VAAYRNAMTVLADUWZS2U'],
  ['attested-usdc-nft-pool', 'CDSHQH4FICZ5RFJG677UP4XTECOAO6IIJTQWB7KN2PQJXGOENEJ6BENT'],
  ['attested-usdc-revenue-pool', 'CACX36IJNDCAZKXEP2DC2ZDJK4QF4R7BHFFXJTSZKJ77K74OP6HAGRQR'],
];
const statuses = await Promise.all(instances.map(async ([name, contract]) => {
  const url = `https://api.stellar.expert/explorer/testnet/contract/${contract}`;
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(20_000) });
    const text = await response.text();
    let body;
    try { body = JSON.parse(text); } catch { body = text; }
    const validationStatus = response.ok && typeof body?.validation?.status === 'string'
      ? body.validation.status : null;
    console.log('EXPLORER_STATUS', name, response.status,
      validationStatus ?? 'NOT_CONFIRMED');
    return { name, contract, url, httpStatus: response.status, validationStatus, body };
  } catch (error) {
    console.log('EXPLORER_STATUS', name, 'REQUEST_FAILED', error.message);
    return { name, contract, url, httpStatus: null, validationStatus: null, error: error.message };
  }
}));
await writeFile(resolve(dirname(input), 'explorer-deployed-status.json'),
  JSON.stringify({
    checkedAt: new Date().toISOString(),
    network: 'testnet',
    run: `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`,
    instances: statuses,
  }, null, 2) + '\n');
if (statuses.some(status => status.validationStatus !== 'verified')) {
  console.log('SOURCE_VERIFICATION_NOT_COMPLETE: inspect explorer-deployed-status.json; no verified-source claim.');
}
