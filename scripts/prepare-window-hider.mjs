// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Builds the library Screenwide loads into the apps it hides on Windows
// (`src-tauri/window-hider`) for the processor the app is built for, and
// puts it in `src-tauri/binaries` for bundling and in `src-tauri/target/debug`
// for `tauri dev`, beside the app, where it is looked for. Cargo decides what
// needs rebuilding, so an unchanged library costs about a second.

import { execFileSync } from "node:child_process";
import { copyFile, mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { arch as hostArch, env, exit, platform } from "node:process";

if (platform !== "win32") {
  console.log("The window hider is only built on Windows");
  exit(0);
}

const triples = {
  arm64: "aarch64-pc-windows-msvc",
  x64: "x86_64-pc-windows-msvc",
};
const architectures = {
  aarch64: "arm64",
  arm64: "arm64",
  x64: "x64",
  x86_64: "x64",
};
const target = triples[architectures[env.TAURI_ENV_ARCH ?? hostArch]];
if (!target) {
  throw new Error(
    `Screenwide does not package the window hider for ${hostArch}`,
  );
}

const targetDirectory = resolve("src-tauri", "target", "window-hider");
execFileSync(
  "cargo",
  ["build", "--release", "--target", target, "--target-dir", targetDirectory],
  { cwd: resolve("src-tauri", "window-hider"), stdio: "inherit" },
);

const built = resolve(
  targetDirectory,
  target,
  "release",
  "screenwide_window_hider.dll",
);
const name = "screenwide-window-hider.dll";
for (const destination of [
  resolve("src-tauri", "binaries", name),
  resolve("src-tauri", "target", "debug", name),
]) {
  await mkdir(dirname(destination), { recursive: true });
  await copyFile(built, destination);
}

console.log(`Prepared the window hider for ${target}`);
