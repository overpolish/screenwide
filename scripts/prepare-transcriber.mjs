// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Builds the helpers, the transcriber, dictation's and dictation's cleanup,
// and puts them where Tauri's externalBin expects them. Cargo decides what needs
// rebuilding, so unchanged helpers cost about a second. The models bundled
// for the transcriber to run, the voice activity model for the microphone
// tools and, on Windows, the text reading models for Text Recognition, are
// fetched once, pinned to a revision and checked against their digests.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { arch as hostArch, env, platform as hostPlatform } from "node:process";
import { dirname, resolve } from "node:path";

// Silero VAD v6.2.0, converted for whisper.cpp by the ggml authors (MIT).
const vadModel = {
  description: "voice activity model",
  name: "silero-vad.bin",
  sha256: "2aa269b785eeb53a82983a20501ddf7c1d9c48e33ab63a41391ac6c9f7fb6987",
  url: "https://huggingface.co/ggml-org/whisper-vad/resolve/9ffd54a1e1ee413ddf265af9913beaf518d1639b/ggml-silero-v6.2.0.bin",
};
// PP-OCRv6 small, PaddlePaddle's own ONNX releases (Apache-2.0): Windows'
// built-in OCR reads small screen text poorly, and macOS keeps Vision. The
// recognition model's configuration lists the characters it reads.
const textModels = [
  {
    description: "text detection model",
    name: "ppocr-detection.onnx",
    sha256: "d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e",
    url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_det_onnx/resolve/28fe5895c24fd108c19eb3e8479f4ab385fbfc62/inference.onnx",
  },
  {
    description: "text recognition model",
    name: "ppocr-recognition.onnx",
    sha256: "5435fd747c9e0efe15a96d0b378d5bd157e9492ed8fd80edf08f30d02fa24634",
    url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/b8f84f0b80c529de40b4fbb3544b84fa7233a513/inference.onnx",
  },
  {
    description: "text recognition configuration",
    name: "ppocr-recognition.yml",
    sha256: "ab078671bb49f06228eadccd34f1bb501e157f7a047095ffb943ba81512c77d1",
    url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/b8f84f0b80c529de40b4fbb3544b84fa7233a513/inference.yml",
  },
];

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
// whisper.cpp's and llama.cpp's Vulkan backends configure their shader
// generator as a nested CMake project deep inside the build directory, and
// CMake caps an object's path at 250 characters while MSVC's linker cannot
// reach files past MAX_PATH, whatever Windows' long path setting. Ninja in
// place of MSBuild's deeper project layout, CMake's hashed SHORT intermediate
// directories and no triple directory when building for the host shorten
// what lies below the target directory, which leaves llama.cpp's deepest
// path about 190 characters below it. On Windows that directory is a short
// one of the same length on every computer, in ProgramData, where any user
// may make a folder: a folder in the checkout or the user's profile varies
// with their depth and the user's name, and llama.cpp's build already fails
// from a runner's local data folder. It is named for the checkout so two
// checkouts keep apart.
const targetDirectory =
  hostPlatform === "win32"
    ? resolve(
        env.ProgramData ?? "C:\\ProgramData",
        "swb",
        createHash("sha256")
          .update(resolve(".").toLowerCase())
          .digest("hex")
          .slice(0, 6),
      )
    : resolve("src-tauri", "target", "stt");
const buildEnv = { ...env };
if (hostPlatform === "win32") {
  const cmakeVersion = execFileSync("cmake", ["--version"], {
    encoding: "utf8",
  }).match(/version (\d+)\.(\d+)/);
  if (
    !cmakeVersion ||
    Number(cmakeVersion[1]) * 100 + Number(cmakeVersion[2]) < 402
  ) {
    throw new Error("Building the transcriber needs CMake 4.2 or later");
  }
  buildEnv.CMAKE_GENERATOR = "Ninja";
  buildEnv.CMAKE_INTERMEDIATE_DIR_STRATEGY = "SHORT";
  try {
    execFileSync("ninja", ["--version"], { stdio: "ignore" });
  } catch {
    // The Build Tools' C++ CMake tools component ships Ninja without putting
    // it on PATH.
    const vswhere = resolve(
      env["ProgramFiles(x86)"] ?? "C:\\Program Files (x86)",
      "Microsoft Visual Studio",
      "Installer",
      "vswhere.exe",
    );
    const ninja = execFileSync(
      vswhere,
      [
        "-latest",
        "-products",
        "*",
        "-find",
        "Common7\\IDE\\CommonExtensions\\Microsoft\\CMake\\Ninja\\ninja.exe",
      ],
      { encoding: "utf8" },
    )
      .split(/\r?\n/)
      .find(Boolean);
    if (!ninja) {
      throw new Error(
        "Building the transcriber needs Ninja: install the C++ CMake tools for Windows component of the Visual Studio Build Tools, or put ninja on PATH",
      );
    }
    // Windows names the variable Path, and a second PATH key would shadow it.
    const pathKey =
      Object.keys(buildEnv).find((key) => key.toUpperCase() === "PATH") ??
      "PATH";
    buildEnv[pathKey] = `${dirname(ninja)};${buildEnv[pathKey] ?? ""}`;
  }
}
// ggml otherwise tunes itself to the processor that builds it, and a release
// is built on GitHub's runners, not on the computers that run it: an
// instruction the runner has and a customer's processor lacks crashes the
// helper there. x64's baseline is AVX2, FMA, F16C and BMI2, no AVX-512, named
// outright rather than left to ggml's defaults: those default off while
// native tuning is on, and a build directory configured that way keeps them
// off in its cache once native tuning is turned off. The ARM baseline is the
// Snapdragon 850's, the oldest processor Windows 11 runs on ARM: dot product
// and half floats, no SVE or i8mm.
const ggmlOptions = {};
if (requestedPlatform === "win32") {
  ggmlOptions.GGML_NATIVE = "OFF";
}
if (target === "x86_64-pc-windows-msvc") {
  for (const option of [
    "GGML_SSE42",
    "GGML_AVX",
    "GGML_AVX2",
    "GGML_FMA",
    "GGML_F16C",
    "GGML_BMI2",
  ]) {
    ggmlOptions[option] = "ON";
  }
}
// MSVC's compiler cannot build Windows ARM64's native parts: whisper.cpp's CPU
// backend refuses it on ARM, and tract's kernels are GNU assembly it cannot
// read. LLVM's clang-cl builds them all instead; it must be on PATH, as it is
// on GitHub's Windows ARM64 runners. The cmake crate's flags replace CMake's
// default /EHsc, which MSVC only warns about losing and clang-cl needs for
// whisper.cpp's exceptions.
if (target === "aarch64-pc-windows-msvc") {
  buildEnv.CC_aarch64_pc_windows_msvc = "clang-cl";
  buildEnv.CXX_aarch64_pc_windows_msvc = "clang-cl";
  buildEnv.CXXFLAGS_aarch64_pc_windows_msvc = "/EHsc";
  ggmlOptions.GGML_CPU_ARM_ARCH = "armv8.2-a+dotprod+fp16";
}
// whisper.cpp's and llama.cpp's build scripts pass every GGML_ variable on
// to CMake; transcribe.cpp's reads none of them and takes its CMake options
// from TRANSCRIBE_CMAKE_ARGS alone, so dictation's helper needs them there or
// it is tuned to the runner.
if (Object.keys(ggmlOptions).length > 0) {
  Object.assign(buildEnv, ggmlOptions);
  buildEnv.TRANSCRIBE_CMAKE_ARGS = Object.entries(ggmlOptions)
    .map(([name, value]) => `-D${name}=${value}`)
    .join(" ");
}
// Node's own architecture can differ from Rust's host under emulation, so
// ask rustc.
const rustHost = execFileSync("rustc", ["-vV"], { encoding: "utf8" }).match(
  /^host: (\S+)$/m,
)?.[1];
const crossCompiling = target !== rustHost;
// The transcriber, dictation's helper and its cleanup helper build
// separately: each engine brings its own copy of ggml, and one program
// cannot hold two.
const build = (feature, program) =>
  execFileSync(
    "cargo",
    [
      "build",
      "--release",
      "--features",
      feature,
      "--bin",
      program,
      ...(crossCompiling ? ["--target", target] : []),
      "--target-dir",
      targetDirectory,
    ],
    { cwd: crate, env: buildEnv, stdio: "inherit" },
  );
build("engine", "screenwide-transcriber");
build("dictation", "screenwide-dictation");
build("cleanup", "screenwide-cleanup");

const extension = requestedPlatform === "win32" ? ".exe" : "";
const builtDirectory = resolve(
  targetDirectory,
  ...(crossCompiling ? [target] : []),
  "release",
);
const built = resolve(builtDirectory, `screenwide-transcriber${extension}`);
for (const program of [
  "screenwide-transcriber",
  "screenwide-dictation",
  "screenwide-cleanup",
]) {
  for (const destination of [
    resolve("src-tauri", "binaries", `${program}-${target}${extension}`),
    // Beside the development build too, where the app looks first, as it
    // does for FFmpeg.
    resolve("src-tauri", "target", "debug", `${program}${extension}`),
  ]) {
    await mkdir(dirname(destination), { recursive: true });
    await copyFile(
      resolve(builtDirectory, `${program}${extension}`),
      destination,
    );
    if (hostPlatform !== "win32") await chmod(destination, 0o755);
  }
}
// The GPU library Studio sound runs through, which ONNX Runtime's build keeps
// out of the helper: beside the development helper, and in binaries, from
// where tauri.macos.conf.json and tauri.windows.conf.json bundle it where the
// installed helper looks.
const gpuLibrary = { darwin: "libwebgpu_dawn.dylib", win32: "DirectML.dll" }[
  requestedPlatform
];
if (gpuLibrary) {
  for (const directory of [
    resolve("src-tauri", "binaries"),
    resolve("src-tauri", "target", "debug"),
  ]) {
    await copyFile(
      resolve(dirname(built), gpuLibrary),
      resolve(directory, gpuLibrary),
    );
  }
}

const sha256 = async (path) =>
  createHash("sha256")
    .update(await readFile(path))
    .digest("hex");
// In binaries, from where the Tauri configuration bundles it, and beside the
// development build, where the app looks when it runs unbundled.
const prepareModel = async ({ description, name, sha256: expected, url }) => {
  const bundled = resolve("src-tauri", "binaries", name);
  if ((await sha256(bundled).catch(() => null)) !== expected) {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(
        `The ${description} download failed with HTTP ${response.status}`,
      );
    }
    const bytes = new Uint8Array(await response.arrayBuffer());
    if (createHash("sha256").update(bytes).digest("hex") !== expected) {
      throw new Error(
        `The downloaded ${description} failed SHA-256 verification`,
      );
    }
    await mkdir(dirname(bundled), { recursive: true });
    await writeFile(bundled, bytes);
  }
  await copyFile(bundled, resolve("src-tauri", "target", "debug", name));
};
await prepareModel(vadModel);
if (requestedPlatform === "win32") {
  for (const model of textModels) await prepareModel(model);
}

console.log(`Prepared the transcriber for ${target}`);
