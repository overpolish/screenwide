// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Runs the built transcriber's speech enhancer on a few seconds of a noisy
// tone. The enhancer runs on tract, whose hand-written kernels are assembled
// per processor (and patched for Windows ARM64, `src-tauri/vendor`), so a
// build can succeed and still compute garbage or crash; this proves the
// kernels run on the machine that built them.
//
//   node scripts/check-transcriber-clean.mjs [--target <triple>]
//
// Run after `pnpm transcriber:prepare`, on a machine of the target's own
// processor.

import { Buffer } from "node:buffer";
import { spawn } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { arch, argv, platform } from "node:process";

const hosts = {
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-arm64": "aarch64-pc-windows-msvc",
  "win32-x64": "x86_64-pc-windows-msvc",
};
const targetFlag = argv.indexOf("--target");
const target =
  targetFlag === -1 ? hosts[`${platform}-${arch}`] : argv[targetFlag + 1];
if (!target) throw new Error(`No transcriber target for ${platform}-${arch}`);
const extension = platform === "win32" ? ".exe" : "";
const transcriber = resolve(
  "src-tauri",
  "binaries",
  `screenwide-transcriber-${target}${extension}`,
);

// The enhancer's own rate (`CLEAN_SAMPLE_RATE`), mono 32-bit floats.
const rate = 48_000;
const seconds = 3;
const input = new Float32Array(rate * seconds);
let seed = 1;
for (let index = 0; index < input.length; index += 1) {
  seed = (seed * 1_103_515_245 + 12_345) % 2_147_483_648;
  const noise = (seed / 2_147_483_648 - 0.5) * 0.2;
  input[index] = 0.3 * Math.sin((2 * Math.PI * 220 * index) / rate) + noise;
}

const rms = (samples) =>
  Math.sqrt(
    samples.reduce((sum, value) => sum + value * value, 0) / samples.length,
  );

const work = await mkdtemp(join(tmpdir(), "screenwide-clean-"));
try {
  const audio = join(work, "in.f32");
  const output = join(work, "out.f32");
  await writeFile(audio, Buffer.from(input.buffer));

  const child = spawn(transcriber, [], { stdio: ["pipe", "pipe", "inherit"] });
  const exited = new Promise((resolveExit, reject) => {
    child.on("error", reject);
    child.on("exit", (code, signal) => resolveExit({ code, signal }));
  });
  const replies = createInterface({ input: child.stdout });
  child.stdin.write(
    `${JSON.stringify({ id: 1, type: "cleanSpeech", audio, output })}\n`,
  );

  const timeout = setTimeout(() => child.kill(), 180_000);
  let answer;
  for await (const line of replies) {
    const reply = JSON.parse(line);
    if (reply.type === "cleaned" || reply.type === "failed") {
      answer = reply;
      break;
    }
  }
  clearTimeout(timeout);
  child.stdin.end();
  const { code, signal } = await exited;

  if (!answer) {
    throw new Error(
      `The transcriber stopped without cleaning (exit ${code ?? signal})`,
    );
  }
  if (answer.type === "failed") {
    throw new Error(`The speech enhancer failed: ${answer.message}`);
  }

  const cleaned = new Float32Array(
    new Uint8Array(await readFile(output)).buffer,
  );
  if (cleaned.length !== input.length) {
    throw new Error(
      `The enhancer wrote ${cleaned.length} samples for ${input.length}`,
    );
  }
  if (!cleaned.every(Number.isFinite)) {
    throw new Error("The enhancer wrote samples that are not numbers");
  }
  // Taking noise out never makes the sound louder; garbage from a broken
  // kernel does.
  if (rms(cleaned) > rms(input) * 1.5) {
    throw new Error(
      `The enhancer made the sound louder (${rms(input)} to ${rms(cleaned)})`,
    );
  }
  console.log(
    `The speech enhancer ran on ${target}: ${cleaned.length} samples, level ${rms(input).toFixed(3)} to ${rms(cleaned).toFixed(3)}`,
  );
} finally {
  await rm(work, { force: true, recursive: true });
}
