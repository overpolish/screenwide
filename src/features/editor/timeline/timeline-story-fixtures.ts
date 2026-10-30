// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { recordedTime, type Span, type TimeMap } from "./timeline-scale";

// Sample recordings behind the timeline parts' stories: drawn frames for
// thumbnails, peaks for a voice, and a squeeze that plays lulls faster.

/** How wide the stories draw the strip, in px. */
export const STORY_TRACK_PX = 640;

/** The shapes a recording comes in: a desktop, a square crop, a phone. */
const FRAME_SIZES = {
  square: { height: 100, width: 100 },
  tall: { height: 160, width: 90 },
  wide: { height: 100, width: 160 },
};

export type StoryFrameShape = keyof typeof FRAME_SIZES;

/** A shape's width over its height, as the recording reports it. */
export const storyFrameAspect = (shape: StoryFrameShape = "wide") =>
  FRAME_SIZES[shape].width / FRAME_SIZES[shape].height;

/** A drawn screen at a moment, as the thumbnail sheet would give it, in
 * `shape`: a title bar, a picture and lines of text, with a sidebar where
 * it is wide enough; its colours turning with `index`. */
export function storyFrame(index: number, shape: StoryFrameShape = "wide") {
  const hue = String(205 + index * 23);
  const { height: h, width: w } = FRAME_SIZES[shape];
  const side = w > h ? 42 : 8;
  const rect = ([x, y, width, height]: number[], light: number, chroma = 12) =>
    `<rect x="${String(x)}" y="${String(y)}" width="${String(width)}" height="${String(height)}" rx="3" fill="hsl(${hue} ${String(chroma)}% ${String(light)}%)"/>`;
  const svg = [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${String(w)} ${String(h)}">`,
    `<rect width="${String(w)}" height="${String(h)}" fill="hsl(${hue} 20% 92%)"/>`,
    `<rect width="${String(w)}" height="10" fill="hsl(${hue} 12% 80%)"/>`,
    w > h ? rect([8, 18, 26, h - 26], 84, 18) : "",
    rect([side, 18, w - side - 8, h * 0.44], 62, 55),
    rect([side, 26 + h * 0.44, (w - side - 8) * 0.6, 8], 70),
    rect([side, 38 + h * 0.44, (w - side - 8) * 0.4, 8], 76),
    "</svg>",
  ].join("");
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

/** Speech-like peaks, the same for the same `seed`: syllables rising and
 * falling, with a breath now and then. */
export function storyPeaks(seed: number, count = 96) {
  return Array.from({ length: count }, (_, index) => {
    const syllable = Math.abs(Math.sin(index * 0.9 + seed * 1.7));
    const phrase = 0.55 + 0.45 * Math.sin(index * 0.13 + seed);
    return index % 23 < 2 ? 0.05 : syllable * phrase;
  });
}

/** A recording `duration` seconds long in which each lull plays in its
 * `plays` seconds instead of its own length, evenly through it; a cut is a
 * lull that plays in none. */
export function squeezedTime(
  duration: number,
  lulls: { end: number; plays: number; start: number }[],
): TimeMap {
  const sorted = [...lulls].sort((first, second) => first.start - second.start);
  const saved = (lull: (typeof sorted)[number]) =>
    lull.end - lull.start - lull.plays;
  const toPlayed = (recorded: number) => {
    let played = recorded;
    for (const lull of sorted) {
      if (recorded <= lull.start) break;
      const through = Math.min(recorded, lull.end) - lull.start;
      played -= through - (through / (lull.end - lull.start)) * lull.plays;
    }
    return played;
  };
  return {
    duration: duration - sorted.reduce((sum, lull) => sum + saved(lull), 0),
    toPlayed,
    toRecorded: (played) => {
      let recorded = played;
      for (const lull of sorted) {
        const from = toPlayed(lull.start);
        if (played <= from) break;
        const length = lull.end - lull.start;
        const through = played - from;
        // Past its end, the whole of what it saves; a cut is always past.
        recorded +=
          through >= lull.plays
            ? length - lull.plays
            : (through / lull.plays) * length - through;
      }
      return recorded;
    },
  };
}

/** A two-minute recording's beats, in recorded seconds; the lulls are the
 * gaps between them. */
export const STORY_BEATS: Span[] = [
  [1, 6],
  [9, 11],
  [12, 19],
  [26, 28],
  [29, 33],
  [41, 47],
  [48, 50],
  [58, 66],
  [68, 70],
  [79, 84],
  [85, 92],
  [97, 99],
  [101, 110],
  [114, 118],
].map(([start, end]) => ({ end, start }));

export const STORY_DURATION = 120;

export const STORY_LULLS: Span[] = STORY_BEATS.slice(1).map((beat, index) => ({
  end: beat.start,
  start: STORY_BEATS[index].end,
}));

const LULL_SECONDS = STORY_LULLS.reduce(
  (sum, lull) => sum + lull.end - lull.start,
  0,
);

/** The most the sample's lulls squeeze by: to a quarter of their length. */
export const STORY_MAX_SQUEEZE = LULL_SECONDS * 0.75;

/** The sample as it plays with its lulls squeezed by `squeeze` seconds,
 * shared between them by their length. */
export const storySqueeze = (squeeze: number) =>
  squeeze <= 0
    ? recordedTime(STORY_DURATION)
    : squeezedTime(
        STORY_DURATION,
        STORY_LULLS.map((lull) => {
          const length = lull.end - lull.start;
          return { ...lull, plays: length - (squeeze * length) / LULL_SECONDS };
        }),
      );
