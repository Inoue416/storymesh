#!/usr/bin/env node

import { validatePackageVersions } from "./npm-packages.mjs";

const expected = process.argv[2];
const version = await validatePackageVersions();
if (expected && version !== expected) {
  throw new Error(`release tag version ${expected} does not match package version ${version}`);
}
console.log(`Cargo and npm package versions match at ${version}.`);
