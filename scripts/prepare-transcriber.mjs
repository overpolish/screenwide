// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Builds the speech-to-text helper and puts it where Tauri's externalBin
// expects it. Cargo decides what needs rebuilding, so an unchanged helper
// costs about a second.

import { execFileSync } from "node:child_process";
import { chmod, copyFile, mkdir } from "node:fs/promises";
import { arch as hostArch, env, platform as hostPlatform } from "node:process";
import { dirname, resolve } from "node:path";

const triples = {
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-arm64": "aarch64-pc-windows-msvc",
  "win32-x64": "x86_64-pc-windows-msvc",
};
const nodeArchitectures = {
  aarch64: "arm64",
  arm64: "arm64",
  x86_64: "x64",
  x64: "x64",
};
const nodePlatforms = {
  darwin: "darwin",
  win32: "win32",
  windows: "win32",
};
const requestedPlatform = nodePlatforms[env.TAURI_ENV_PLATFORM ?? hostPlatform];
const requestedArch = nodeArchitectures[env.TAURI_ENV_ARCH ?? hostArch];
const target = triples[`${requestedPlatform}-${requestedArch}`];
if (!target) {
  throw new Error(
    `Screenwide does not package the transcriber for ${requestedPlatform}-${requestedArch}`,
  );
}

const crate = resolve("src-tauri", "transcriber");
// whisper.cpp builds through MSBuild on Windows, which fails once its
// try-compile paths pass MAX_PATH. A short directory, and no triple directory
// when building for the host, keeps those paths well inside the limit.
const targetDirectory = resolve("src-tauri", "target", "stt");
// Node's own architecture can differ from Rust's host under emulation, so
// ask rustc.
const rustHost = execFileSync("rustc", ["-vV"], { encoding: "utf8" }).match(
  /^host: (\S+)$/m,
)?.[1];
const crossCompiling = target !== rustHost;
execFileSync(
  "cargo",
  [
    "build",
    "--release",
    "--features",
    "engine",
    ...(crossCompiling ? ["--target", target] : []),
    "--target-dir",
    targetDirectory,
  ],
  { cwd: crate, stdio: "inherit" },
);

const extension = requestedPlatform === "win32" ? ".exe" : "";
const built = resolve(
  targetDirectory,
  ...(crossCompiling ? [target] : []),
  "release",
  `screenwide-transcriber${extension}`,
);
const place = async (destination) => {
  await mkdir(dirname(destination), { recursive: true });
  await copyFile(built, destination);
  if (hostPlatform !== "win32") await chmod(destination, 0o755);
};
await place(
  resolve(
    "src-tauri",
    "binaries",
    `screenwide-transcriber-${target}${extension}`,
  ),
);
// Beside the development build too, where the app looks first, as it does
// for FFmpeg.
await place(
  resolve("src-tauri", "target", "debug", `screenwide-transcriber${extension}`),
);

console.log(`Prepared the transcriber for ${target}`);
