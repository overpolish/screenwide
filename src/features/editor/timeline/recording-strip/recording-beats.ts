// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineBeat } from "../beat-clip/beat-label";
import { type Span } from "../timeline-scale";

// The beats of a real recording and the lulls between them, found in what
// was read from it: what was said, grouped into lines; the clicks, the
// typing and the shortcuts pressed; the moments flagged while recording;
// and where the picture went new. Each beat keeps a little of what led up
// to it and what came of it. Where two would overlap, the earlier gives
// way, unless that would leave it too short to see, when the later starts
// after it instead. Beats and lulls tile the recording: a gap too short to
// be a lull goes to the beat before it.

/** What a real recording holds to find its beats in, in recorded seconds. */
export type RecordingReadings = {
  clicks: { at: number; count: number }[];
  duration: number;
  moments: { at: number; color: string; index: number; name: string }[];
  /** Where the picture was not at rest. */
  picture: (Span & { state: "moving" | "new" })[];
  /** Each shortcut from its press until its keys leave the picture. */
  shortcuts: { at: number; end: number; id: string; keys: string[] }[];
  /** When text was typed; the key file says no more than that. */
  typing: number[];
  words: (Span & { text: string })[];
};

export type RecordingBeat = { beat: TimelineBeat; id: string; span: Span };

export type RecordingLull = {
  id: string;
  span: Span;
  /** Where the picture moves by itself in it. */
  moving?: Span[];
};

/** How long a pause between words, in seconds, ends a line. */
const LINE_GAP_S = 0.7;
/** How long a pause between typing, in seconds, still holds one run. */
const TYPING_GAP_S = 2;
/** The shortest beat kept, in seconds. */
const LEAST_BEAT_S = 0.3;
/** The shortest gap between beats, in seconds, that is a lull. */
const LEAST_LULL_S = 0.3;

/** How long before each thing done the pointer or hands set off, and after
 * it the work settles, in seconds. */
const AROUND = {
  click: [1, 0.5],
  keys: [0.4, 0.5],
  moment: [4, 2],
  type: [0.6, 1.2],
} as const;

const idOf = (kind: string, at: number) =>
  `${kind}-${String(Math.round(at * 1000))}`;

/** The id of the speech beat found for a line said from `start`. */
export const speechBeatId = (start: number) => idOf("speech", start);

const around = (at: number, [before, after]: readonly [number, number]) => ({
  end: at + after,
  start: at - before,
});

/** The words, grouped into lines: a line ends at a long enough pause or
 * where a sentence does. */
export function linesOf(words: RecordingReadings["words"]) {
  const lines: (Span & {
    text: string;
    words: { at: number; word: string }[];
  })[] = [];
  let last: (typeof words)[number] | null = null;
  for (const word of [...words].sort((one, other) => one.start - other.start)) {
    const text = word.text.trim();
    if (!text) continue;
    const line = lines[lines.length - 1] as (typeof lines)[number] | undefined;
    const isNew =
      !line ||
      !last ||
      word.start - last.end > LINE_GAP_S ||
      /[.!?]$/.test(last.text.trim());
    const said = { at: word.start, word: text };
    if (isNew)
      lines.push({ end: word.end, start: word.start, text, words: [said] });
    else {
      line.end = word.end;
      line.text = `${line.text} ${text}`;
      line.words.push(said);
    }
    last = word;
  }
  return lines;
}

/** Runs of typing, each from its first event to its last. */
function typingRuns(typing: number[]): Span[] {
  const runs: Span[] = [];
  for (const at of [...typing].sort((one, other) => one - other)) {
    const run = runs[runs.length - 1] as Span | undefined;
    if (run && at - run.end <= TYPING_GAP_S) run.end = at;
    else runs.push({ end: at, start: at });
  }
  return runs;
}

/** Every beat the readings hold, as found, overlapping where they do. */
function foundBeats(readings: RecordingReadings): RecordingBeat[] {
  return [
    ...linesOf(readings.words).map((line) => ({
      beat: { kind: "speech", line: line.text } as const,
      id: speechBeatId(line.start),
      span: { end: line.end, start: line.start },
    })),
    ...readings.clicks.map(({ at }) => ({
      beat: { kind: "click" } as const,
      id: idOf("click", at),
      span: around(at, AROUND.click),
    })),
    ...typingRuns(readings.typing).map((run) => ({
      beat: { kind: "type" } as const,
      id: idOf("type", run.start),
      span: {
        end: run.end + AROUND.type[1],
        start: run.start - AROUND.type[0],
      },
    })),
    ...readings.shortcuts.map(({ at, id, keys }) => ({
      beat: { keys, kind: "keys" } as const,
      id: `keys-${id}`,
      span: around(at, AROUND.keys),
    })),
    ...readings.moments.map(({ at, color, index, name }) => ({
      beat: { color, kind: "moment", name } as const,
      id: `moment-${String(index)}`,
      span: around(at, AROUND.moment),
    })),
    ...readings.picture
      .filter(({ state }) => state === "new")
      .map(({ end, start }) => ({
        beat: { kind: "screen" } as const,
        id: idOf("screen", start),
        span: { end: Math.max(end, start + 1), start },
      })),
  ];
}

/** The beats, inside the recording and one after another. */
function laidOut(beats: RecordingBeat[], duration: number) {
  const kept: RecordingBeat[] = [];
  const sorted = beats
    .map((beat) => ({
      ...beat,
      span: {
        end: Math.min(beat.span.end, duration),
        start: Math.max(beat.span.start, 0),
      },
    }))
    .sort((one, other) => one.span.start - other.span.start);
  for (const beat of sorted) {
    const last = kept[kept.length - 1] as RecordingBeat | undefined;
    let { end, start } = beat.span;
    if (last && start < last.span.end) {
      if (start - last.span.start >= LEAST_BEAT_S) last.span.end = start;
      else start = last.span.end;
    }
    end = Math.max(end, start);
    if (end - start >= LEAST_BEAT_S)
      kept.push({ ...beat, span: { end, start } });
  }
  return kept;
}

/** What is left of `readings` with the screen's track out of the export:
 * the voice and the moments. The clicks, the typing, the shortcuts and the
 * picture were all read off the screen, so the lulls are the gaps between
 * what was said and flagged, and none moves by itself. */
export const withoutScreen = (
  readings: RecordingReadings,
): RecordingReadings => ({
  ...readings,
  clicks: [],
  picture: [],
  shortcuts: [],
  typing: [],
});

/** A recording's beats and the lulls between them. */
export type RecordingBeats = { beats: RecordingBeat[]; lulls: RecordingLull[] };

/** The recording's beats and the lulls between them, tiling it. */
export function recordingBeats(readings: RecordingReadings): RecordingBeats {
  const { duration } = readings;
  const beats = laidOut(foundBeats(readings), duration);
  const lulls: RecordingLull[] = [];
  const moving = readings.picture.filter(({ state }) => state === "moving");
  const addLull = (start: number, end: number) => {
    const span = { end, start };
    const inside = moving
      .map((each) => ({
        end: Math.min(each.end, end),
        start: Math.max(each.start, start),
      }))
      .filter((each) => each.end > each.start);
    lulls.push({
      id: idOf("lull", start),
      span,
      ...(inside.length > 0 ? { moving: inside } : {}),
    });
  };
  let at = 0;
  beats.forEach((beat, index) => {
    const gap = beat.span.start - at;
    if (gap >= LEAST_LULL_S) addLull(at, beat.span.start);
    else if (index === 0) beat.span.start = 0;
    else beats[index - 1].span.end = beat.span.start;
    at = beat.span.end;
  });
  if (duration - at >= LEAST_LULL_S) addLull(at, duration);
  else if (beats.length > 0) beats[beats.length - 1].span.end = duration;
  return { beats, lulls };
}
