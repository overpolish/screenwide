// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Bundles a release's x64 and ARM64 Store packages into the one upload
// Partner Center takes: both architectures in an `.msixbundle`, zipped as an
// `.msixupload`. Once a release has shipped as a bundle, every later
// submission must carry one. Prints the upload's path.
//
//   node scripts/bundle-msix.mjs <folder of .msix> <tag, such as v1.3.0>
//
// Windows only, as it needs the Windows SDK's makeappx.

import { execFileSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { mkdir, rename, rm } from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import { argv } from "node:process";

import { findSdkTool } from "./windows-sdk.mjs";

const [folder, tag] = argv.slice(2);
if (!folder || !tag) {
  throw new Error("Usage: node scripts/bundle-msix.mjs <folder> <tag>");
}
// The Store wants four numbers and a last one of 0, as each package has.
if (!/^v\d+\.\d+\.\d+$/.test(tag)) {
  throw new Error(`A Store bundle needs a plain release tag, not ${tag}`);
}
const version = `${tag.slice(1)}.0`;

const packages = readdirSync(folder).filter((name) => name.endsWith(".msix"));
if (packages.length !== 2) {
  throw new Error(
    `Expected the x64 and ARM64 packages, found ${String(packages.length)}`,
  );
}

const out = resolve("msix-upload");
await rm(out, { force: true, recursive: true });
await mkdir(out, { recursive: true });
const bundle = join(out, `Screenwide_${version}.msixbundle`);
execFileSync(
  findSdkTool("makeappx"),
  ["bundle", "/d", resolve(folder), "/p", bundle, "/bv", version, "/o"],
  { stdio: ["ignore", "ignore", "inherit"] },
);
// The upload is the bundle in a zip. Windows' own tar (bsdtar) writes a zip
// for a name ending in `.zip`, which is then given the upload's extension.
const zip = join(out, `Screenwide_${version}.zip`);
const upload = join(out, `Screenwide_${version}.msixupload`);
execFileSync("tar", ["-a", "-c", "-f", zip, "-C", out, basename(bundle)], {
  stdio: ["ignore", "ignore", "inherit"],
});
await rename(zip, upload);
console.log(upload);
