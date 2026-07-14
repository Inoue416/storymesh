#!/usr/bin/env node

import { chmod, cp, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

export const platformPackages = [
  { name: "storymesh-darwin-arm64", os: "darwin", cpu: "arm64", binary: "storymesh" },
  { name: "storymesh-darwin-x64", os: "darwin", cpu: "x64", binary: "storymesh" },
  { name: "storymesh-linux-arm64", os: "linux", cpu: "arm64", binary: "storymesh" },
  { name: "storymesh-linux-x64", os: "linux", cpu: "x64", binary: "storymesh" },
  { name: "storymesh-win32-x64", os: "win32", cpu: "x64", binary: "storymesh.exe" },
];

async function readJson(file) {
  return JSON.parse(await readFile(file, "utf8"));
}

export async function validatePackageVersions(root = repositoryRoot) {
  const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
  const version = cargo.match(/^version = "([^"]+)"$/m)?.[1];
  if (!version) throw new Error("Cargo.toml package version was not found");

  const cli = await readJson(path.join(root, "npm/storymesh/package.json"));
  if (cli.version !== version) {
    throw new Error(`npm/storymesh version ${cli.version} does not match Cargo version ${version}`);
  }

  const expectedDependencies = Object.fromEntries(
    platformPackages.map(({ name }) => [name, version]),
  );
  if (JSON.stringify(cli.optionalDependencies) !== JSON.stringify(expectedDependencies)) {
    throw new Error("storymesh optionalDependencies must contain every platform package at the same version");
  }

  for (const platform of platformPackages) {
    const manifest = await readJson(
      path.join(root, "npm/platforms", platform.name, "package.json"),
    );
    if (manifest.name !== platform.name || manifest.version !== version) {
      throw new Error(`${platform.name} name and version must match ${platform.name}@${version}`);
    }
    if (manifest.os?.[0] !== platform.os || manifest.cpu?.[0] !== platform.cpu) {
      throw new Error(`${platform.name} has incorrect os/cpu metadata`);
    }
    const expectedLibc = platform.os === "linux" ? "glibc" : undefined;
    if (manifest.libc?.[0] !== expectedLibc) {
      throw new Error(`${platform.name} has incorrect libc metadata`);
    }
  }

  return version;
}

export async function preparePackages({
  root = repositoryRoot,
  artifactsDirectory,
  outputDirectory,
}) {
  const version = await validatePackageVersions(root);
  await mkdir(outputDirectory, { recursive: false });

  for (const platform of platformPackages) {
    const destination = path.join(outputDirectory, platform.name);
    await cp(path.join(root, "npm/platforms", platform.name), destination, { recursive: true });
    await mkdir(path.join(destination, "bin"));
    const binary = path.join(destination, "bin", platform.binary);
    await cp(path.join(artifactsDirectory, platform.name, platform.binary), binary);
    if (platform.os !== "win32") await chmod(binary, 0o755);
    await cp(path.join(root, "README.md"), path.join(destination, "README.md"));
    await cp(path.join(root, "LICENSE"), path.join(destination, "LICENSE"));
  }

  const cliDestination = path.join(outputDirectory, "storymesh");
  await cp(path.join(root, "npm/storymesh"), cliDestination, { recursive: true });
  await cp(path.join(root, "README.md"), path.join(cliDestination, "README.md"));
  await cp(path.join(root, "LICENSE"), path.join(cliDestination, "LICENSE"));
  return { version, directories: [...platformPackages.map(({ name }) => name), "storymesh"] };
}

async function run() {
  const [artifactsDirectory, outputDirectory] = process.argv.slice(2);
  if (!artifactsDirectory || !outputDirectory) {
    throw new Error("usage: scripts/npm-packages.mjs ARTIFACTS_DIRECTORY OUTPUT_DIRECTORY");
  }
  const result = await preparePackages({
    artifactsDirectory: path.resolve(artifactsDirectory),
    outputDirectory: path.resolve(outputDirectory),
  });
  await writeFile(
    path.join(path.resolve(outputDirectory), "packages.json"),
    `${JSON.stringify(result, null, 2)}\n`,
  );
  console.log(`Prepared ${result.directories.length} npm packages at version ${result.version}.`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  run().catch((error) => {
    console.error(`npm package preparation failed: ${error.message}`);
    process.exitCode = 1;
  });
}
