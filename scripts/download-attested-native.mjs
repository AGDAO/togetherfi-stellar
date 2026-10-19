// Fetch public candidate bytes and cryptographically verify their exact origin.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const [sha, destination] = process.argv.slice(2);
if (!/^[a-f0-9]{40}$/.test(sha || '') || !destination) {
  throw new Error('Usage: download-attested-native.mjs <source-commit> <new-output-dir>');
}
const OUT = resolve(destination);
const REPO = 'AGDAO/togetherfi-stellar';
const BUILDER = '4a554f0e75c245cf4d3ea5f98f3a653d51edcd18';
const WORKFLOW = 'stellar-expert/soroban-build-workflow/.github/workflows/release.yml';
await mkdir(OUT, { recursive: true });
const headers = {
  Accept: 'application/vnd.github+json',
  ...(process.env.GITHUB_WORKFLOW_TOKEN
    ? { Authorization: `Bearer ${process.env.GITHUB_WORKFLOW_TOKEN}` } : {}),
};
async function api(path) {
  const response = await fetch(`https://api.github.com/repos/${REPO}${path}`, {
    headers, signal: AbortSignal.timeout(30_000),
  });
  if (!response.ok) throw new Error(`GitHub ${response.status}: ${path}`);
  return response.json();
}
const result = {
  network_authorization: 'testnet-only', source_commit: sha,
  source_repository: REPO, builder_commit: BUILDER, builder_workflow: WORKFLOW,
  human_source_review: 'pending; build attestation is not reviewer approval',
  initialization: 'deployer-preimage-v1', wasm_sha256: {}, packages: {},
};
for (const name of ['campaign_escrow', 'contributor_pool', 'reputation_anchor']) {
  const packageName = name.replaceAll('_', '-');
  const tag = `testnet-candidate-${sha}_contracts_${name}_${packageName}_pkg0.1.0_cli22.0.1`;
  const ref = await api(`/git/ref/tags/${tag}`);
  if (ref.object.type !== 'commit' || ref.object.sha !== sha) {
    throw new Error('Candidate tag does not identify the attested source commit');
  }
  const release = await api(`/releases/tags/${tag}`);
  const asset = release.assets.find(a => a.name === `${packageName}_v0.1.0.wasm`);
  if (!asset) throw new Error('Candidate release missing native WASM: ' + name);
  const file = resolve(OUT, `${name}.wasm`);
  try { await readFile(file); throw new Error('Do not overwrite retained WASM evidence'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  const response = await fetch(asset.browser_download_url, { signal: AbortSignal.timeout(60_000) });
  if (!response.ok) throw new Error('WASM asset download failed: ' + response.status);
  const wasm = Buffer.from(await response.arrayBuffer());
  if (wasm.subarray(0, 4).toString('hex') !== '0061736d') throw new Error('Not a WASM artifact');
  await writeFile(file, wasm, { flag: 'wx' });
  const digest = createHash('sha256').update(wasm).digest('hex');
  const verification = execFileSync('gh', [
    'attestation', 'verify', file, '--repo', REPO,
    '--signer-workflow', WORKFLOW, '--signer-digest', BUILDER,
    '--source-digest', sha, '--deny-self-hosted-runners', '--format', 'json',
  ], {
    env: { ...process.env, ...(process.env.GITHUB_WORKFLOW_TOKEN
      ? { GH_TOKEN: process.env.GITHUB_WORKFLOW_TOKEN } : {}) },
    encoding: 'utf8', timeout: 120_000, maxBuffer: 4_000_000,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  const verified = JSON.parse(verification);
  if (!Array.isArray(verified) || verified.length === 0) throw new Error('No verified attestation');
  await writeFile(resolve(OUT, `${name}.verification.json`), verification, { flag: 'wx' });
  result.wasm_sha256[name] = digest;
  result.packages[name] = {
    file, sha256: digest, release: release.html_url, asset: asset.browser_download_url,
    attestation_verified: true, verification_file: `${name}.verification.json`,
  };
  console.log(`VERIFIED ${name} sha256:${digest}`);
}
await writeFile(resolve(OUT, 'attested-build.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx' });
console.log('ATTESTED_BUILD_READY ' + resolve(OUT, 'attested-build.json'));
