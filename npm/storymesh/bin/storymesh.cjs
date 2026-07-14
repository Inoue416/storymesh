#!/usr/bin/env node

const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { packageForPlatform } = require("../lib/platform.cjs");

let platformPackage;
try {
  platformPackage = packageForPlatform(process.platform, process.arch);
} catch (error) {
  console.error(`storymesh: ${error.message}`);
  process.exit(1);
}

let packageJson;
try {
  packageJson = require.resolve(`${platformPackage}/package.json`);
} catch {
  console.error(
    `storymesh: the optional package ${platformPackage} is missing. ` +
      "Reinstall without --no-optional, and make sure this platform is supported.",
  );
  process.exit(1);
}

const executable = process.platform === "win32" ? "storymesh.exe" : "storymesh";
const binary = path.join(path.dirname(packageJson), "bin", executable);
const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(`storymesh: failed to start ${binary}: ${result.error.message}`);
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 1);
