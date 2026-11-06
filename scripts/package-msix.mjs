// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Packs a Windows release build as the Microsoft Store package (MSIX). Tauri
// makes no MSIX, so this lays the build out as the NSIS installer does: the
// app beside its bundled programs, with the resources the Tauri configs name
// at the paths they map to. It adds the Store logos and the manifest
// (`src-tauri/msix/AppxManifest.xml`), then packs the folder with the Windows
// SDK's makeappx. The Store signs the package when it is submitted.
//
//   node scripts/package-msix.mjs [--target <triple>]
//
// Run after `tauri build --target <triple>`. The staged folder is kept: in
// Windows' developer mode, `Add-AppxPackage -Register <folder>\AppxManifest.xml`
// installs it unsigned, to test it as the Store would install it.

import { execFileSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { arch as hostArch, argv, env, exit, platform } from "node:process";

if (platform !== "win32") {
  console.log("The Microsoft Store package is only made on Windows");
  exit(0);
}

const architectures = {
  "aarch64-pc-windows-msvc": "arm64",
  "x86_64-pc-windows-msvc": "x64",
};
const targetFlag = argv.indexOf("--target");
const target =
  targetFlag === -1
    ? hostArch === "arm64"
      ? "aarch64-pc-windows-msvc"
      : "x86_64-pc-windows-msvc"
    : argv[targetFlag + 1];
const architecture = architectures[target];
if (!architecture) {
  throw new Error(`Screenwide has no Store package for ${target}`);
}

const tauri = resolve("src-tauri");
const readJson = async (path) => JSON.parse(await readFile(path, "utf8"));
const config = await readJson(join(tauri, "tauri.conf.json"));
const windowsConfig = await readJson(join(tauri, "tauri.windows.conf.json"));

// The Store wants four numbers and a last one of 0.
const version = config.version;
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  throw new Error(`A Store package needs a plain version, not ${version}`);
}
const packageVersion = `${version}.0`;

const release = join(tauri, "target", target, "release");
const stage = join(tauri, "target", target, "msix", "Screenwide");
await rm(stage, { force: true, recursive: true });
await mkdir(stage, { recursive: true });

const copy = async (from, to) => {
  if (!existsSync(from)) throw new Error(`${from} is missing from the build`);
  await mkdir(dirname(to), { recursive: true });
  await cp(from, to);
};

await copy(join(release, "screenwide.exe"), join(stage, "screenwide.exe"));

// Bundled programs are kept with the target in their names and installed
// without it, beside the app.
for (const program of config.bundle.externalBin) {
  const name = basename(program);
  await copy(
    join(tauri, `${program}-${target}.exe`),
    join(stage, `${name}.exe`),
  );
}

const resources = {
  ...config.bundle.resources,
  ...windowsConfig.bundle?.resources,
};
for (const [from, to] of Object.entries(resources)) {
  await copy(join(tauri, from), join(stage, to));
}

const logos = ["Square150x150Logo.png", "Square44x44Logo.png", "StoreLogo.png"];
for (const logo of logos) {
  await copy(join(tauri, "icons", logo), join(stage, "Assets", logo));
}

const manifest = (
  await readFile(join(tauri, "msix", "AppxManifest.xml"), "utf8")
)
  .replace("{{VERSION}}", packageVersion)
  .replace("{{ARCHITECTURE}}", architecture);
await writeFile(join(stage, "AppxManifest.xml"), manifest);

// The newest Windows SDK's makeappx, for the machine this runs on, unless
// MAKEAPPX names one.
function findMakeappx() {
  if (env.MAKEAPPX) return env.MAKEAPPX;
  const kits = "C:\\Program Files (x86)\\Windows Kits\\10\\bin";
  const host = hostArch === "arm64" ? "arm64" : "x64";
  const versions = existsSync(kits)
    ? readdirSync(kits)
        .filter((entry) => /^10\.\d+\.\d+\.\d+$/.test(entry))
        .sort((a, b) => a.localeCompare(b, undefined, { numeric: true }))
        .reverse()
    : [];
  for (const sdk of versions) {
    for (const folder of [host, "x64"]) {
      const tool = join(kits, sdk, folder, "makeappx.exe");
      if (existsSync(tool)) return tool;
    }
  }
  throw new Error("makeappx was not found; install the Windows SDK");
}

const output = join(
  release,
  "bundle",
  "msix",
  `Screenwide_${version}_${architecture}.msix`,
);
await mkdir(dirname(output), { recursive: true });
execFileSync(findMakeappx(), ["pack", "/d", stage, "/p", output, "/o"], {
  stdio: "inherit",
});
console.log(`Packed ${output}`);
console.log(`Staged ${stage}`);
