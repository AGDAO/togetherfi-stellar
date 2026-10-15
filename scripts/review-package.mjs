// Explicit export boundary for the standalone Stellar review repository.
// Historical optional application components must not reappear via a recursive copy.
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
export const CONTRACTS = ["campaign_escrow", "contributor_pool", "reputation_anchor"];
const SINGLE_FILES = [
  ".gitignore", ".github/workflows/test.yml", "LICENSE", "README.md",
  "UPSTREAM_COMMIT", "package.json", "package-lock.json",
  "docs/proposed-ci-workflow.yml", "docs/testnet-acceptance.md",
  "docs/testnet-deployment-evidence.md", "docs/testnet-signing.md",
  "docs/testnet-signer-rotation.md", "docs/review-handoff.md",
  "docs/future-work-allocation.md",
  "docs/initialization-hardening.md",
  "RELEASE.md",
  "evidence/testnet-escrow-v2/manifest.json",
  "evidence/testnet-escrow-v2/complete-campaign-1.xdr.txt",
  "evidence/testnet-signer-rotation/request.json",
  "evidence/testnet-signer-rotation/replace-ace-with-zyad.xdr.txt",
  "evidence/testnet-signer-rotation/ayoub-signed.xdr.txt",
  "evidence/testnet-signer-rotation/king-signed.xdr.txt",
  "evidence/testnet-signer-rotation/quorum-signed.xdr.txt",
  "evidence/testnet-signer-rotation/confirmation.json",
  "docs/testnet-zyad-king.md",
  "docs/testnet-zyad-ayoub.md",
];
const SKIP = new Set(["target", "test_snapshots", "node_modules", ".git", ".cargo-ok"]);

function walk(root, directory) {
  return readdirSync(join(root, directory), { withFileTypes: true }).flatMap(entry => {
    if (SKIP.has(entry.name)) return [];
    if (entry.isSymbolicLink()) throw new Error(`Symlink not allowed in review export: ${entry.name}`);
    const path = `${directory}/${entry.name}`;
    return entry.isDirectory() ? walk(root, path) : entry.isFile() ? [path] : [];
  });
}

export function reviewPaths(root = ROOT) {
  return [...SINGLE_FILES, ...CONTRACTS.flatMap(name => [
    `contracts/${name}/Cargo.toml`, `contracts/${name}/Cargo.lock`,
    ...walk(root, `contracts/${name}/src`),
  ]), ...walk(root, "scripts"), ...walk(root, "soroban-env-host-patch"),
  ...walk(root, "evidence/testnet-zyad-king").filter(path =>
    /\/(manifest\.json|complete-campaign-3\.xdr\.txt|(?:zyad|king|quorum)-signed\.xdr\.txt|preparation-unfunded-[0-9]+\.json)$/.test(path)),
  ...walk(root, "evidence/testnet-zyad-ayoub").filter(path =>
    /\/(manifest\.json|complete-campaign-4\.xdr\.txt|(?:zyad|ayoub|quorum)-signed\.xdr\.txt)$/.test(path)),
  ].sort();
}

export function publicLockfile(text) {
  const lock = JSON.parse(text);
  for (const pkg of Object.values(lock.packages)) {
    if (pkg.resolved?.startsWith("http://package-firewall.replit.internal/npm/")) {
      pkg.resolved = pkg.resolved.replace(
        "http://package-firewall.replit.internal/npm/", "https://registry.npmjs.org/");
    }
    if (pkg.resolved && !pkg.resolved.startsWith("https://registry.npmjs.org/")) {
      throw new Error("Unexpected package registry in public review lockfile");
    }
  }
  return JSON.stringify(lock, null, 2) + "\n";
}

export function reviewFiles(root = ROOT) {
  return reviewPaths(root).map(path => {
    const input = readFileSync(join(root, path), "utf8");
    return { path, content: path === "package-lock.json" ? publicLockfile(input) : input };
  });
}

if (process.argv[1] && relative(ROOT, process.argv[1]) === "scripts/review-package.mjs") {
  if (process.argv.includes("--paths")) console.log(reviewPaths().join("\n"));
  else if (process.argv.includes("--json")) console.log(JSON.stringify(reviewFiles()));
  else throw new Error("Use --paths or --json. This exporter never signs or submits transactions.");
}
