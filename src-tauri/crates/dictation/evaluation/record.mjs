// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Records each case of `cases.json` with the macOS voice it names, its
// pieces a pause apart, into `audio/<id>.m4a`: 16 kHz mono, what dictation
// hears. Run on a Mac with FFmpeg on the PATH:
//   node src-tauri/crates/dictation/evaluation/record.mjs
// A case already recorded is left as it is; delete its file to record it
// again.

import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const { cases } = JSON.parse(
  readFileSync(path.join(here, "cases.json"), "utf8"),
);
const audio = path.join(here, "audio");
mkdirSync(audio, { recursive: true });

/** Silence before the first piece, between pieces and after the last, in
 * seconds: a pause long enough to set a spoken command apart. */
const LEAD = 0.4;
const PAUSE = 1.3;
const TAIL = 1.2;

const scratch = mkdtempSync(path.join(tmpdir(), "dictation-cases-"));
try {
  for (const { id, say, voice } of cases) {
    const out = path.join(audio, `${id}.m4a`);
    if (existsSync(out)) continue;
    const pieces = say.map((text, index) => {
      const piece = path.join(scratch, `${id}-${String(index)}.aiff`);
      execFileSync("say", ["-v", voice, "-o", piece, text]);
      return piece;
    });
    // Each piece followed by its pause, the first led in by a little silence.
    const inputs = pieces.flatMap((piece) => ["-i", piece]);
    const chain = pieces
      .map((_, index) => {
        const after = index === pieces.length - 1 ? TAIL : PAUSE;
        const before = index === 0 ? LEAD * 1000 : 0;
        return `[${String(index)}:a]aresample=16000,aformat=channel_layouts=mono,adelay=${String(before)},apad=pad_dur=${String(after)}[p${String(index)}]`;
      })
      .join(";");
    const joined = pieces.map((_, index) => `[p${String(index)}]`).join("");
    execFileSync("ffmpeg", [
      "-loglevel",
      "error",
      "-y",
      ...inputs,
      "-filter_complex",
      `${chain};${joined}concat=n=${String(pieces.length)}:v=0:a=1[out]`,
      "-map",
      "[out]",
      "-ar",
      "16000",
      "-ac",
      "1",
      "-c:a",
      "aac",
      "-b:a",
      "32k",
      out,
    ]);
    console.log(`recorded ${id}`);
  }
} finally {
  rmSync(scratch, { force: true, recursive: true });
}
