#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? "");
if (!process.argv[2]) {
  console.error("usage: scripts/publish-npm-packages.mjs PREPARED_PACKAGES_DIRECTORY");
  process.exit(2);
}

const { version, directories } = JSON.parse(await readFile(path.join(root, "packages.json"), "utf8"));

for (const directory of directories) {
  const manifest = JSON.parse(await readFile(path.join(root, directory, "package.json"), "utf8"));
  if (manifest.version !== version) throw new Error(`${manifest.name} has an unexpected version`);

  const spec = `${manifest.name}@${version}`;
  const existing = spawnSync("npm", ["view", spec, "version", "--json"], { encoding: "utf8" });
  if (existing.status === 0) {
    console.log(`${spec} is already published; skipping.`);
    continue;
  }
  if (!`${existing.stdout}\n${existing.stderr}`.includes("E404")) {
    throw new Error(`could not check ${spec}: ${existing.stderr || existing.stdout}`);
  }

  console.log(`Publishing ${spec}...`);
  const published = spawnSync("npm", ["publish", path.join(root, directory)], { stdio: "inherit" });
  if (published.status !== 0) throw new Error(`npm publish failed for ${spec}`);
}
