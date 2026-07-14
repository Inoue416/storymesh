const assert = require("node:assert/strict");
const { chmodSync, cpSync, mkdtempSync, mkdirSync, rmSync, writeFileSync } = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const test = require("node:test");
const { packageForPlatform } = require("../storymesh/lib/platform.cjs");

test(
  "the launcher forwards arguments and the binary exit status",
  { skip: process.platform === "win32" },
  () => {
    const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "storymesh-launcher-test-"));
    try {
      const cli = path.join(temporaryRoot, "cli");
      cpSync(path.join(__dirname, "../storymesh"), cli, { recursive: true });

      const packageName = packageForPlatform(process.platform, process.arch);
      const platformPackage = path.join(cli, "node_modules", packageName);
      mkdirSync(path.join(platformPackage, "bin"), { recursive: true });
      writeFileSync(path.join(platformPackage, "package.json"), JSON.stringify({ name: packageName }));

      const binary = path.join(platformPackage, "bin", "storymesh");
      writeFileSync(
        binary,
        `#!${process.execPath}\nconsole.log(JSON.stringify(process.argv.slice(2))); process.exit(7);\n`,
      );
      chmodSync(binary, 0o755);

      const result = spawnSync(process.execPath, [path.join(cli, "bin/storymesh.cjs"), "check", "a b"], {
        encoding: "utf8",
      });
      assert.equal(result.status, 7);
      assert.deepEqual(JSON.parse(result.stdout), ["check", "a b"]);
    } finally {
      rmSync(temporaryRoot, { recursive: true, force: true });
    }
  },
);
