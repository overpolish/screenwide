// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RECORDING_S } from "../editor-data";
import { type TimeMap } from "../model/timeline-model";

// Where a moment falls along the strip, as a fraction of its width. The
// edited time comes from the time map (cuts and squeeze). Still time is
// drawn at a fraction of its length, so the beats, where the work happened,
// have room for their pictures even in a recording that is mostly waiting;
// its springs say it is packed. The lens then swells time under the pointer
// the way the Dock swells icons. There is no zoom: the lens is the zoom.

/** How much of its length still time is drawn at. */
const STILL_WEIGHT = 0.3;

/** How much the lens magnifies at its focus, less one: 3 is four times.
 * Gentle enough that what is aimed at moves slowly under the pointer. */
const DISTORTION = 3;

/** Where the lens is, in recorded seconds, and how far it has swelled. */
export type Lens = { focus: number; strength: number };

export type StripScale = {
  /** The recorded moment under a fraction of the strip's width. */
  toSeconds: (x: number) => number;
  /** Fraction of the strip's width, 0 to 1, for a recorded moment. */
  toX: (seconds: number) => number;
};

export function stripScale({
  lens,
  map,
  range = [0, RECORDING_S],
}: {
  map: TimeMap;
  lens?: Lens;
  range?: readonly [number, number];
}): StripScale {
  // Display units: each piece's edited length, still pieces weighted down.
  const { pieces } = map;
  const lengths = pieces.map(
    (piece) =>
      (map.toTime(piece.end) - map.toTime(piece.start)) *
      (piece.beat || piece.isSpeech ? 1 : STILL_WEIGHT),
  );
  const starts = lengths.reduce<number[]>(
    (all, value, index) => [...all, all[index] + value],
    [0],
  );
  const display = (seconds: number) => {
    const index = pieces.findIndex((piece) => seconds < piece.end);
    if (index < 0) return starts[pieces.length];
    const piece = pieces[index];
    const into = Math.max(0, seconds - piece.start) / (piece.end - piece.start);
    return starts[index] + into * lengths[index];
  };
  const from = display(range[0]);
  const span = Math.max(display(range[1]) - from, 1e-6);
  const fraction = (seconds: number) => (display(seconds) - from) / span;
  const original = (u: number) => {
    const target = from + u * span;
    const index = pieces.findIndex(
      (_, at) => lengths[at] > 0 && target < starts[at] + lengths[at],
    );
    if (index < 0) return range[1];
    const piece = pieces[index];
    const into = (target - starts[index]) / lengths[index];
    return piece.start + into * (piece.end - piece.start);
  };
  if (!lens || lens.strength < 0.01) {
    return {
      toSeconds: (x) => original(Math.min(Math.max(x, 0), 1)),
      toX: fraction,
    };
  }
  // Sarkar and Brown's graphical fisheye: the focus stays where it is, so
  // the moment under the pointer stays under it, and the strip stretches on
  // either side of it toward the ends, which stay put. Both directions have
  // an exact formula, so scrubbing reads the same lens it sees.
  const d = DISTORTION * lens.strength;
  const p = Math.min(Math.max(fraction(lens.focus), 0), 1);
  const grow = (t: number) => ((d + 1) * t) / (d * t + 1);
  const shrink = (y: number) => y / (d + 1 - d * y);
  const side = (value: number, bend: (t: number) => number) =>
    value >= p
      ? p + (1 - p) * bend((value - p) / Math.max(1 - p, 1e-6))
      : p - p * bend((p - value) / Math.max(p, 1e-6));
  return {
    toSeconds: (x) => original(side(Math.min(Math.max(x, 0), 1), shrink)),
    toX: (seconds) => side(Math.min(Math.max(fraction(seconds), 0), 1), grow),
  };
}

/** The least width, in px, of anything on the strip that can be pressed. */
const MIN_HIT_PX = 24;

/** A span's place in px, reaching past its drawn width when that is too
 * narrow to press; `pad` is how far it reaches on each side. */
export const reach = (
  scale: StripScale,
  { end, start }: { end: number; start: number },
  width: number,
) => {
  const x = scale.toX(start) * width;
  const px = Math.max(0, (scale.toX(end) - scale.toX(start)) * width);
  const pad = Math.max(0, (MIN_HIT_PX - px) / 2);
  return {
    pad,
    px,
    style: { insetInlineStart: x - pad, width: px + pad * 2 },
  };
};

/** A span's place along the strip, as a style. */
export const placed = (scale: StripScale, start: number, end: number) => ({
  insetInlineStart: `${String(scale.toX(start) * 100)}%`,
  width: `${String(Math.max(0, scale.toX(end) - scale.toX(start)) * 100)}%`,
});

/** A moment's place along the strip, as a style. */
export const at = (scale: StripScale, seconds: number) => ({
  insetInlineStart: `${String(scale.toX(seconds) * 100)}%`,
});
