#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { platformPackages, preparePackages } from "./npm-packages.mjs";

const temporaryRoot = await mkdtemp(path.join(os.tmpdir(), "storymesh-npm-check-"));

try {
  const artifacts = path.join(temporaryRoot, "artifacts");
  await mkdir(artifacts);
  for (const platform of platformPackages) {
    const directory = path.join(artifacts, platform.name);
    await mkdir(directory);
    await writeFile(path.join(directory, platform.binary), "test binary\n");
  }

  const output = path.join(temporaryRoot, "packages");
  const prepared = await preparePackages({ artifactsDirectory: artifacts, outputDirectory: output });
  for (const packageDirectory of prepared.directories) {
    const packagePath = path.join(output, packageDirectory);
    const packed = JSON.parse(
      execFileSync("npm", ["pack", "--dry-run", "--json", packagePath], {
        encoding: "utf8",
        env: { ...process.env, npm_config_cache: path.join(temporaryRoot, "npm-cache") },
      }),
    )[0];
    const files = new Set(packed.files.map(({ path: file }) => file));
    for (const required of ["package.json", "README.md", "LICENSE"]) {
      if (!files.has(required)) throw new Error(`${packageDirectory} tarball is missing ${required}`);
    }
    if (packageDirectory === "storymesh") {
      for (const required of ["bin/storymesh.cjs", "lib/platform.cjs"]) {
        if (!files.has(required)) throw new Error(`storymesh tarball is missing ${required}`);
      }
    } else {
      const manifest = JSON.parse(await readFile(path.join(packagePath, "package.json"), "utf8"));
      const binary = manifest.os[0] === "win32" ? "bin/storymesh.exe" : "bin/storymesh";
      if (!files.has(binary)) throw new Error(`${packageDirectory} tarball is missing ${binary}`);
    }
  }
  console.log(`Validated ${prepared.directories.length} npm package tarballs.`);
} finally {
  await rm(temporaryRoot, { recursive: true, force: true });
}
