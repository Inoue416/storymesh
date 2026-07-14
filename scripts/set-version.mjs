#!/usr/bin/env node

import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { platformPackages } from "./npm-packages.mjs";

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("usage: scripts/set-version.mjs MAJOR.MINOR.PATCH[-PRERELEASE]");
  process.exit(2);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

async function replace(file, pattern, replacement) {
  const contents = await readFile(file, "utf8");
  if (!pattern.test(contents)) throw new Error(`version field was not found in ${file}`);
  await writeFile(file, contents.replace(pattern, replacement));
}

await replace(path.join(root, "Cargo.toml"), /^version = "[^"]+"$/m, `version = "${version}"`);
await replace(
  path.join(root, "Cargo.lock"),
  /(\[\[package\]\]\nname = "storymesh"\nversion = ")[^"]+("\n)/,
  (_match, prefix, suffix) => `${prefix}${version}${suffix}`,
);

const manifests = [
  path.join(root, "npm/storymesh/package.json"),
  ...platformPackages.map(({ name }) => path.join(root, "npm/platforms", name, "package.json")),
];

for (const manifestPath of manifests) {
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  manifest.version = version;
  if (manifest.name === "storymesh") {
    manifest.optionalDependencies = Object.fromEntries(
      platformPackages.map(({ name }) => [name, version]),
    );
  }
  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
}

console.log(`Updated Cargo and npm package versions to ${version}.`);
