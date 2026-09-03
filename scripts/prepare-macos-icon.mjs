// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { copyFile, mkdir, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { exit, platform } from "node:process";
import { fileURLToPath } from "node:url";

if (platform !== "darwin") exit(0);

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const workspaceDirectory = resolve(scriptDirectory, "..");
const iconAsset = join(workspaceDirectory, "src-tauri/icons/Screenwide.icon");
const bundledAsset = join(workspaceDirectory, "src-tauri/icons/Assets.car");
const version = spawnSync(
  "xcrun",
  ["actool", "--version", "--output-format=human-readable-text"],
  { encoding: "utf8" },
);
const majorVersion = Number(
  `${version.stdout ?? ""}\n${version.stderr ?? ""}`.match(
    /short-bundle-version:\s*(\d+)/,
  )?.[1],
);

// Xcode before 26 cannot compile Icon Composer packages. Assets.car is kept in
// the repository for those machines; icon.icns remains the runtime fallback.
if (version.status !== 0 || majorVersion < 26) {
  console.warn("Xcode 26 is unavailable; keeping the existing macOS app icon");
  exit(0);
}

const buildDirectory = await mkdtemp(join(tmpdir(), "screenwide-icon-"));
const outputDirectory = join(buildDirectory, "out");
await mkdir(outputDirectory);

try {
  const result = spawnSync(
    "xcrun",
    [
      "actool",
      iconAsset,
      "--compile",
      outputDirectory,
      "--output-format",
      "human-readable-text",
      "--notices",
      "--warnings",
      "--output-partial-info-plist",
      join(outputDirectory, "assetcatalog_generated_info.plist"),
      "--app-icon",
      "Screenwide",
      "--include-all-app-icons",
      "--enable-on-demand-resources",
      "NO",
      "--development-region",
      "en",
      "--target-device",
      "mac",
      "--minimum-deployment-target",
      "26.0",
      "--platform",
      "macosx",
    ],
    { stdio: "inherit" },
  );
  if (result.status !== 0) {
    throw new Error(`actool exited with status ${result.status ?? "unknown"}`);
  }
  await copyFile(join(outputDirectory, "Assets.car"), bundledAsset);
} finally {
  await rm(buildDirectory, { force: true, recursive: true });
}
