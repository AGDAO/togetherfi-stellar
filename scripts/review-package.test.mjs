import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { CONTRACTS, ROOT, reviewFiles, publicLockfile } from "./review-package.mjs";

test("review snapshot includes only the three native contract packages", () => {
  const files = reviewFiles();
  assert.equal(new Set(files.map(f => f.path)).size, files.length);
  const packages = [...new Set(files.filter(f => f.path.startsWith("contracts/"))
    .map(f => f.path.split("/")[1]))].sort();
  assert.deepEqual(packages, [...CONTRACTS].sort());
  assert.ok(!files.some(f => /(^|\/)(vendor|evm-layerzero|node_modules|target|test_snapshots)\//.test(f.path)));
  assert.ok(!files.some(f => /layerzero_funding_inbox|layerzero_receipt_adapter/.test(f.path)));
  const packageJson = JSON.parse(files.find(f => f.path === "package.json").content);
  assert.equal(packageJson.name, "togetherfi-stellar-validation");
  assert.ok(!files.find(f => f.path === "package-lock.json").content.includes("package-firewall.replit.internal"));
  const workflow = files.find(f => f.path === ".github/workflows/test.yml").content;
  assert.deepEqual([...workflow.matchAll(/- contract: (\w+)/g)].map(m => m[1]).sort(), packages);
  for (const name of packages) {
    assert.ok(files.some(f => f.path === `contracts/${name}/Cargo.lock`));
  }
});

test("public lockfile changes registry URLs only, not dependency identity or integrity", () => {
  const input = JSON.parse(readFileSync(join(ROOT, "package-lock.json"), "utf8"));
  const output = JSON.parse(publicLockfile(JSON.stringify(input)));
  for (const [path, before] of Object.entries(input.packages)) {
    const after = output.packages[path];
    const { resolved: oldUrl, ...oldRest } = before;
    const { resolved: newUrl, ...newRest } = after;
    assert.deepEqual(newRest, oldRest);
    if (oldUrl) assert.ok(newUrl.startsWith("https://registry.npmjs.org/"));
  }
  assert.throws(() => publicLockfile(JSON.stringify({ packages: {
    bad: { resolved: "https://unapproved.invalid/package.tgz" },
  } })));
});
