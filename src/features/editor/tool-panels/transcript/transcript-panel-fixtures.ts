// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToolPanelModel } from "../tool-panel-selection";
import { TranscriptRange } from "../tool-panel-transcript";

import { firstIndex, TranscriptWord } from "./transcript-words";

const SAMPLE = `Welcome back. In this recording I'm going to show you how the new export settings work, and um, why they matter for longer videos. First, open the editor and pick the the recording you want to share. You'll see the timeline along the bottom, with your microphone and the system audio on separate tracks. Uh, so the first thing I usually do is trim the start, because there's almost always a few seconds of me getting set up. Next, take a look at the panel on the right. Studio sound cleans up the voice, and Remove silences takes out the long pauses, so you don't have to hunt for them yourself. When you're happy with it, press export, choose a size, and damn, that's it. The file lands in your recordings folder, ready to send. Thanks for watching, and I'll see you in the next one.`;

/** Words the sample marks as heard with little confidence. */
const UNSURE = new Set(["um,", "Uh,", "hunt"]);
/** Sentences the speaker pauses after for longer, moving to a new step. */
const STEP_ENDS = new Set(["videos.", "share.", "up.", "yourself."]);

/** The sample timed as speech: longer words take longer, a sentence's end
 * leaves a pause after it that shows, a comma only a breath that does not,
 * and a new step a longer pause. The recording starts and ends with a
 * silence long enough to show. */
export const sampleTranscriptWords: TranscriptWord[] = (() => {
  let at = 1_800;
  return SAMPLE.split(" ").map((text) => {
    const startMs = at;
    const endMs = startMs + Math.min(180 + text.length * 45, 700);
    const pause = STEP_ENDS.has(text)
      ? 2_200
      : /[.!?]$/u.test(text)
        ? 700
        : text.endsWith(",")
          ? 200
          : 90;
    at = endMs + pause;
    return {
      endMs,
      probability: UNSURE.has(text) ? 0.38 : 0.94,
      startMs,
      text,
    };
  });
})();

export const sampleTranscriptDurationMs =
  sampleTranscriptWords[sampleTranscriptWords.length - 1].endMs + 3_000;

/** What Remove silences would cut from the sample, as the app plans it for
 * a speaker who usually pauses for under half a second: every silence of a
 * second or more, less the beat it keeps after speech and before it. */
export const sampleTranscriptSilences: TranscriptRange[] = (() => {
  const words = sampleTranscriptWords;
  const edges = [
    { endMs: words[0].startMs, startMs: 0 },
    ...words.slice(1).map((word, index) => ({
      endMs: word.startMs,
      startMs: words[index].endMs,
    })),
    {
      endMs: sampleTranscriptDurationMs,
      startMs: words[words.length - 1].endMs,
    },
  ];
  return edges
    .filter((gap) => gap.endMs - gap.startMs >= 1_000)
    .map((gap) => ({
      endMs:
        gap.endMs === sampleTranscriptDurationMs ? gap.endMs : gap.endMs - 150,
      startMs: gap.startMs === 0 ? 0 : gap.startMs + 250,
    }));
})();

/** The stretch the sample's word `text` takes, its first time. */
export const sampleWordRange = (text: string): TranscriptRange => {
  const word = sampleTranscriptWords.find((each) => each.text === text);
  return { endMs: word?.endMs ?? 0, startMs: word?.startMs ?? 0 };
};

/** The Transcripts model, at the size of Whisper large-v3-turbo in q8_0. */
export const sampleTranscriptsModel = (
  status: ToolPanelModel["status"],
  progress = 0,
): ToolPanelModel => ({ progress, sizeBytes: 874_188_075, status });

/** `ranges` with `added` cut too, merged where they meet. */
export function addRanges(
  ranges: readonly TranscriptRange[],
  added: readonly TranscriptRange[],
) {
  const merged: TranscriptRange[] = [];
  for (const range of [...ranges, ...added].sort(
    (a, b) => a.startMs - b.startMs,
  )) {
    const last = merged[merged.length - 1] as TranscriptRange | undefined;
    if (last && range.startMs <= last.endMs)
      last.endMs = Math.max(last.endMs, range.endMs);
    else merged.push({ ...range });
  }
  return merged;
}

/** `ranges` with `taken` brought back. */
export function subtractRanges(
  ranges: readonly TranscriptRange[],
  taken: readonly TranscriptRange[],
) {
  return taken.reduce<TranscriptRange[]>(
    (left, cut) =>
      left.flatMap((range) =>
        cut.endMs <= range.startMs || cut.startMs >= range.endMs
          ? [range]
          : [
              { endMs: cut.startMs, startMs: range.startMs },
              { endMs: range.endMs, startMs: cut.endMs },
            ].filter((part) => part.endMs > part.startMs),
      ),
    [...ranges],
  );
}

/** Where playback at `ms` really is, once it has jumped every cut. */
export function skipRemoved(ms: number, removed: readonly TranscriptRange[]) {
  const after = firstIndex(removed.length, (i) => removed[i].startMs > ms);
  const within = after > 0 ? removed[after - 1] : undefined;
  return within && ms < within.endMs ? within.endMs : ms;
}
