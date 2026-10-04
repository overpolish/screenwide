// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// wgpu compiles WGSL through DXC on Windows. Its default, FXC, takes minutes
// on the larger shaders, so the pinned Microsoft release is placed beside the
// app: in `src-tauri/binaries` for bundling and debug runs of tests, and in
// `src-tauri/target/debug` for `tauri dev`. From this release on, DXC signs
// shaders itself, so the validator (`dxil.dll`) is not needed. The build
// script precompiles the canvas shader with `dxc.exe`, which is placed beside
// the library it loads and is not bundled.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { copyFile, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { arch, exit, platform } from "node:process";

const archiveUrl =
  "https://github.com/microsoft/DirectXShaderCompiler/releases/download/v1.8.2502/dxc_2025_02_20.zip";
const archiveSha256 =
  "70b1913a1bfce4a3e1a5311d16246f4ecdf3a3e613abec8aa529e57668426f85";
const libraries = {
  arm64: "add120337487e273af0d649446aa61ff0b90475dc1a31ffb2774e870d4ec6830",
  x64: "f79943b02f73b621421b58a569fd6ee120d0023fb6bfa23844ecad6b00d6a295",
};
const executables = {
  arm64: "e0a7f8fd076350aa442f44820822d5154c318cfa0d416ba1eb21c2829872bd34",
  x64: "ae4bf100b8c64ab03cdc0ee5996d5f212e96c59d6d37340359342d714ac6c004",
};
const licenses = ["LICENSE-LLVM.txt", "LICENSE-MIT.txt"];

const sha256 = (path) =>
  new Promise((resolveDigest, reject) => {
    const hash = createHash("sha256");
    createReadStream(path)
      .on("error", reject)
      .on("data", (chunk) => hash.update(chunk))
      .on("end", () => resolveDigest(hash.digest("hex")));
  });

const matches = (path, expected) =>
  sha256(path)
    .then((digest) => digest === expected)
    .catch(() => false);

if (platform !== "win32") {
  console.log("DXC is only bundled on Windows");
  exit(0);
}
const librarySha256 = libraries[arch];
const executableSha256 = executables[arch];
if (!librarySha256 || !executableSha256) {
  throw new Error(`Screenwide does not package DXC for Windows ${arch}`);
}

const binaries = resolve("src-tauri", "binaries");
const library = join(binaries, "dxcompiler.dll");
const executable = join(binaries, "dxc.exe");
const licenseTargets = licenses.map((name) => join(binaries, `DXC-${name}`));
const prepared =
  (await matches(library, librarySha256)) &&
  (await matches(executable, executableSha256)) &&
  (
    await Promise.all(
      licenseTargets.map((path) => sha256(path).then(Boolean, () => false)),
    )
  ).every(Boolean);

if (!prepared) {
  const workDirectory = await mkdtemp(join(tmpdir(), "screenwide-dxc-"));
  try {
    const archive = join(workDirectory, "dxc.zip");
    const response = await fetch(archiveUrl);
    if (!response.ok) {
      throw new Error(`DXC download failed with HTTP ${response.status}`);
    }
    await writeFile(archive, new Uint8Array(await response.arrayBuffer()));
    if ((await sha256(archive)) !== archiveSha256) {
      throw new Error("The downloaded DXC archive failed SHA-256 verification");
    }
    const extracted = join(workDirectory, "extracted");
    await mkdir(extracted);
    execFileSync("tar", ["-xf", archive, "-C", extracted], {
      stdio: "inherit",
    });
    const source = join(extracted, "bin", arch, "dxcompiler.dll");
    if (!(await matches(source, librarySha256))) {
      throw new Error("The extracted DXC library failed SHA-256 verification");
    }
    const sourceExecutable = join(extracted, "bin", arch, "dxc.exe");
    if (!(await matches(sourceExecutable, executableSha256))) {
      throw new Error("The extracted DXC compiler failed SHA-256 verification");
    }
    await mkdir(binaries, { recursive: true });
    await copyFile(source, library);
    await copyFile(sourceExecutable, executable);
    await Promise.all(
      licenses.map((name, index) =>
        copyFile(join(extracted, name), licenseTargets[index]),
      ),
    );
  } finally {
    await rm(workDirectory, { force: true, recursive: true });
  }
}

const debugLibrary = resolve("src-tauri", "target", "debug", "dxcompiler.dll");
if (!(await matches(debugLibrary, librarySha256))) {
  await mkdir(dirname(debugLibrary), { recursive: true });
  await copyFile(library, debugLibrary);
}

console.log(`Prepared bundled DXC for Windows ${arch}`);
