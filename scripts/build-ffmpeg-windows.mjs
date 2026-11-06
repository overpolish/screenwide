// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import {
  copyFile,
  mkdir,
  mkdtemp,
  readdir,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { arch, argv, exit, platform } from "node:process";

// BtbN deletes its daily builds after about two weeks and keeps the build from
// the end of each month, so a pin must be a month-end `autobuild-*` release.
const builds = {
  arm64: {
    archiveUrl:
      "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-10-31-12-59/ffmpeg-n8.1.3-27-g8dca1586f4-winarm64-gpl-8.1.zip",
    archiveSha256:
      "40029da82e02abf0d12491e23666e710b7a8943d13501594cf38e6135b95efba",
    executableSha256:
      "0fa09f6cd5582e7a9404312087203f2f921b4020459ddb69ab1669502a03972c",
  },
  x64: {
    archiveUrl:
      "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-10-31-12-59/ffmpeg-n8.1.3-27-g8dca1586f4-win64-gpl-8.1.zip",
    archiveSha256:
      "401dffcf14c301082e1fc03b0bc876b92ea676bfc521fc56e80a1537d137034f",
    executableSha256:
      "c46215f8889634396f0ac0acac738ef2145175e5b6caf28b3daa8ac9a9980c5f",
  },
};

const sha256 = (path) =>
  new Promise((resolveDigest, reject) => {
    const hash = createHash("sha256");
    createReadStream(path)
      .on("error", reject)
      .on("data", (chunk) => hash.update(chunk))
      .on("end", () => resolveDigest(hash.digest("hex")));
  });

const findFfmpeg = async (directory) => {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      const nested = await findFfmpeg(path);
      if (nested) return nested;
    } else if (entry.name === "ffmpeg.exe") {
      return path;
    }
  }
};

if (platform !== "win32") {
  throw new Error("The Screenwide Windows FFmpeg builder must run on Windows");
}
if (argv.length !== 3) {
  throw new Error("usage: build-ffmpeg-windows.mjs OUTPUT");
}

const build = builds[arch];
if (!build) {
  throw new Error(`Screenwide does not package FFmpeg for Windows ${arch}`);
}
const { archiveSha256, archiveUrl, executableSha256 } = build;
const output = resolve(argv[2]);
if (
  await sha256(output)
    .then((digest) => digest === executableSha256)
    .catch(() => false)
) {
  exit(0);
}

const workDirectory = await mkdtemp(join(tmpdir(), "screenwide-ffmpeg-"));
try {
  const archive = join(workDirectory, "ffmpeg.zip");
  const response = await fetch(archiveUrl);
  if (!response.ok) {
    throw new Error(`FFmpeg download failed with HTTP ${response.status}`);
  }
  await writeFile(archive, new Uint8Array(await response.arrayBuffer()));
  if ((await sha256(archive)) !== archiveSha256) {
    throw new Error(
      "The downloaded FFmpeg archive failed SHA-256 verification",
    );
  }

  const extracted = join(workDirectory, "extracted");
  await mkdir(extracted);
  execFileSync("tar", ["-xf", archive, "-C", extracted], { stdio: "inherit" });
  const source = await findFfmpeg(extracted);
  if (!source || (await sha256(source)) !== executableSha256) {
    throw new Error(
      "The extracted FFmpeg executable failed SHA-256 verification",
    );
  }
  await mkdir(dirname(output), { recursive: true });
  await copyFile(source, output);
} finally {
  await rm(workDirectory, { force: true, recursive: true });
}
