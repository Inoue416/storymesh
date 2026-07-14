const assert = require("node:assert/strict");
const test = require("node:test");
const { packageForPlatform } = require("../storymesh/lib/platform.cjs");

const supported = [
  ["darwin", "arm64", "storymesh-darwin-arm64"],
  ["darwin", "x64", "storymesh-darwin-x64"],
  ["linux", "arm64", "storymesh-linux-arm64"],
  ["linux", "x64", "storymesh-linux-x64"],
  ["win32", "x64", "storymesh-win32-x64"],
];

for (const [platform, arch, expected] of supported) {
  test(`${platform} ${arch} selects ${expected}`, () => {
    assert.equal(packageForPlatform(platform, arch), expected);
  });
}

test("an unsupported platform has an actionable error", () => {
  assert.throws(() => packageForPlatform("freebsd", "x64"), /unsupported platform: freebsd x64/);
});
