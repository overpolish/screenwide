// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Records a person reading each sentence of `voice-cases.json` into
// `voice/<id>.m4a`, 16 kHz mono, what dictation hears. The recordings stay
// on this computer: `voice/` is not committed. Run on a Mac with FFmpeg on
// the PATH, from a terminal allowed to use the microphone:
//   node src-tauri/crates/dictation/evaluation/record-voice.mjs [microphone]
// `microphone` is a number from FFmpeg's list, shown when it is left out. A
// sentence already recorded is skipped; delete its file to record it again.

import { spawn, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
} from "node:fs";
import path from "node:path";
import { createInterface } from "node:readline/promises";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const { cases } = JSON.parse(
  readFileSync(path.join(here, "voice-cases.json"), "utf8"),
);
const voice = path.join(here, "voice");
mkdirSync(voice, { recursive: true });

const microphone = process.argv[2];
if (microphone === undefined) {
  const listed = spawnSync(
    "ffmpeg",
    ["-hide_banner", "-f", "avfoundation", "-list_devices", "true", "-i", ""],
    { encoding: "utf8" },
  );
  const devices = listed.stderr
    .split("\n")
    .slice(
      listed.stderr
        .split("\n")
        .findIndex((line) => line.includes("audio devices")) + 1,
    )
    .filter((line) => /\[\d+\]/.test(line))
    .map((line) => line.replace(/^.*?\] /, ""));
  console.log("Microphones:");
  for (const device of devices) console.log(`  ${device}`);
  console.log("Run again with the number of the one to record from.");
  process.exit(0);
}

const terminal = createInterface({
  input: process.stdin,
  output: process.stdout,
});
// The recording under way, if one is, and the file it is written to. Recorded
// under another name until finished, so a recording cut off part-way, which
// FFmpeg leaves unreadable, is not taken as done.
let recording;
let unfinished;
// However the script ends, FFmpeg must not go on holding the microphone.
process.on("exit", () => {
  recording?.kill("SIGKILL");
  if (unfinished) rmSync(unfinished, { force: true });
});
const quit = () => process.exit(130);
terminal.on("SIGINT", quit);
process.on("SIGINT", quit);
process.on("SIGTERM", quit);
process.on("SIGHUP", quit);
const left = cases.filter(
  ({ id }) => !existsSync(path.join(voice, `${id}.m4a`)),
);
console.log(
  `${String(left.length)} sentences to read. For each, press Return, read it as you would dictate it, then press Return again.`,
);
for (const [index, { id, say }] of left.entries()) {
  console.log(`\n${String(index + 1)}/${String(left.length)}  ${say}`);
  await terminal.question("Return to start ");
  const finished = path.join(voice, `${id}.m4a`);
  unfinished = path.join(voice, `${id}.part.m4a`);
  recording = spawn(
    "ffmpeg",
    [
      "-loglevel",
      "error",
      "-y",
      "-f",
      "avfoundation",
      "-i",
      `:${microphone}`,
      "-ar",
      "16000",
      "-ac",
      "1",
      "-c:a",
      "aac",
      "-b:a",
      "48k",
      unfinished,
    ],
    { stdio: ["pipe", "inherit", "inherit"] },
  );
  await terminal.question("Recording. Return to stop ");
  // FFmpeg finishes the file properly when asked to quit on its input.
  recording.stdin.write("q");
  await new Promise((resolve) => recording.on("close", resolve));
  renameSync(unfinished, finished);
  recording = undefined;
  unfinished = undefined;
}
terminal.close();
console.log(`\nDone. The recordings are in ${voice}.`);
