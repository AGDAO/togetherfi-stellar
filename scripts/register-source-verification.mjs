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
