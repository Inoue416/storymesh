const PACKAGES = new Map([
  ["darwin-arm64", "storymesh-darwin-arm64"],
  ["darwin-x64", "storymesh-darwin-x64"],
  ["linux-arm64", "storymesh-linux-arm64"],
  ["linux-x64", "storymesh-linux-x64"],
  ["win32-x64", "storymesh-win32-x64"],
]);

function packageForPlatform(platform, arch) {
  const key = `${platform}-${arch}`;
  const packageName = PACKAGES.get(key);
  if (!packageName) {
    throw new Error(`unsupported platform: ${platform} ${arch}`);
  }
  return packageName;
}

module.exports = { packageForPlatform };
