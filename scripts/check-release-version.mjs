// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { readFile } from "node:fs/promises";
import { argv } from "node:process";

const readJson = async (path) => JSON.parse(await readFile(path, "utf8"));
const packageJson = await readJson("package.json");
const tauriConfig = await readJson("src-tauri/tauri.conf.json");
const cargoToml = await readFile("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const versions = new Map([
  ["package.json", packageJson.version],
  ["src-tauri/Cargo.toml", cargoVersion],
  ["src-tauri/tauri.conf.json", tauriConfig.version],
]);
const expected = packageJson.version;

for (const [path, version] of versions) {
  if (version !== expected) {
    throw new Error(`${path} is ${String(version)}; expected ${expected}`);
  }
}

const tag = argv[2];
if (tag && tag !== `v${expected}`) {
  throw new Error(`release tag ${tag} does not match app version v${expected}`);
}

// A version is the year and month its release ships in, then which release
// of that month it is, from zero: 26.11.0, 26.11.1, 26.12.0. The month has no
// leading zero, which a version may not carry. A prerelease adds a suffix
// (26.12.0-rc.1) and may be cut ahead of the month it is for.
const SCHEME = /^(\d{2})\.([1-9]|1[0-2])\.(?:0|[1-9]\d*)(-[0-9A-Za-z.-]+)?$/;
const scheme = SCHEME.exec(expected);
if (!scheme) {
  throw new Error(`${expected} is not year.month.release, such as 26.11.0`);
}
const [, year, month, prerelease] = scheme;
if (!prerelease) {
  // The release workflow runs in UTC, so the month is UTC's here too.
  const now = new Date();
  const current = `${now.getUTCFullYear() % 100}.${now.getUTCMonth() + 1}`;
  if (`${Number(year)}.${month}` !== current) {
    throw new Error(
      `${expected} is not this month's: a release now is ${current}.N`,
    );
  }
}

console.log(`Release version ${expected} is consistent`);
