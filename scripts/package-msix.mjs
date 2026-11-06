// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Packs a Windows release build as the Microsoft Store package (MSIX). Tauri
// makes no MSIX, so this lays the build out as the NSIS installer does: the
// app beside its bundled programs, with the resources the Tauri configs name
// at the paths they map to. It adds the Store logos, the taskbar's unplated
// logos (`src-tauri/msix/Assets`) and the manifest
// (`src-tauri/msix/AppxManifest.xml`), indexes them with the Windows SDK's
// makepri and packs the folder with its makeappx. The Store signs the package
// when it is submitted.
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
// Without unplated sizes of the small logo, the taskbar and Start draw it
// shrunk on a tile of the accent colour.
await cp(join(tauri, "msix", "Assets"), join(stage, "Assets"), {
  recursive: true,
});

const manifest = (
  await readFile(join(tauri, "msix", "AppxManifest.xml"), "utf8")
)
  .replaceAll("{{VERSION}}", packageVersion)
  .replaceAll("{{ARCHITECTURE}}", architecture);
await writeFile(join(stage, "AppxManifest.xml"), manifest);

// The newest Windows SDK's copy of `name` (makeappx, makepri), for the machine
// this runs on, unless the variable of its name in capitals names one.
function findSdkTool(name) {
  const override = env[name.toUpperCase()];
  if (override) return override;
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
      const tool = join(kits, sdk, folder, `${name}.exe`);
      if (existsSync(tool)) return tool;
    }
  }
  throw new Error(`${name} was not found; install the Windows SDK`);
}

// Windows finds a logo's sizes through the package's resource index, and
// ignores any file it does not list. Only the logos are indexed, from a copy
// of `Assets` laid out as in the package: the rest of the package is plain
// files, some with dots makepri would read as qualifiers
// (`whisper.cpp-MIT.txt`). The copy and the index's config stay out of the
// package.
const makepri = findSdkTool("makepri");
const indexed = join(dirname(stage), "resources");
await rm(indexed, { force: true, recursive: true });
await cp(join(stage, "Assets"), join(indexed, "Assets"), { recursive: true });
const priConfig = join(dirname(stage), "priconfig.xml");
execFileSync(
  makepri,
  ["createconfig", "/cf", priConfig, "/dq", "en-US", "/pv", "10.0.0", "/o"],
  { stdio: "inherit" },
);
execFileSync(
  makepri,
  [
    "new",
    "/pr",
    indexed,
    "/cf",
    priConfig,
    "/mn",
    join(stage, "AppxManifest.xml"),
    "/of",
    join(stage, "resources.pri"),
    "/o",
  ],
  { stdio: "inherit" },
);

const output = join(
  release,
  "bundle",
  "msix",
  `Screenwide_${version}_${architecture}.msix`,
);
await mkdir(dirname(output), { recursive: true });
execFileSync(
  findSdkTool("makeappx"),
  ["pack", "/d", stage, "/p", output, "/o"],
  {
    stdio: "inherit",
  },
);
console.log(`Packed ${output}`);
console.log(`Staged ${stage}`);
