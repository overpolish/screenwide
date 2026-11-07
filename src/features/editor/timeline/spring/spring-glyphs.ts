// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Where glyphs sit on a spring's coil, and the room the coil leaves round
// each so the glyph reads clear of it.

/** Half the clear room, in px, the wire leaves round a glyph on it: the
 * glyph's width with a little air. */
const GAP_HALF_PX = 7;
/** How far, in px, the wire fades out on each side of that room, as a
 * scroll area fades its content at an edge rather than cutting it. */
const GAP_FADE_PX = 8;
/** How much of the wire, in px, one glyph takes out of it. */
const GAP_PX = (GAP_HALF_PX + GAP_FADE_PX) * 2;

/** Where glyphs go on a wire `width` px long, for things at `wanted` px
 * along it. Things nearer each other than a glyph's room share one glyph,
 * in the middle of them, so glyphs never crowd out the wire between them;
 * where those would still take out more than half the wire, one glyph
 * stands in the middle of them all, so the lull still reads as a spring. */
export function glyphPlaces(wanted: number[], width: number): number[] {
  if (wanted.length === 0) return [];
  const sorted = [...wanted].sort((first, second) => first - second);
  const runs: [number, number][] = [];
  for (const at of sorted) {
    const last = runs[runs.length - 1] as [number, number] | undefined;
    if (last && at - last[1] < GAP_PX) last[1] = at;
    else runs.push([at, at]);
  }
  if (runs.length > 1 && runs.length * GAP_PX > width / 2)
    return [(sorted[0] + sorted[sorted.length - 1]) / 2];
  return runs.map(([first, last]) => (first + last) / 2);
}

/** A mask that fades the wire out round each of `gaps`, in px along it and
 * in order. */
export function gapMask(gaps: number[]) {
  const px = (at: number) => `${String(at)}px`;
  const stops = gaps.flatMap((at) => [
    `#000 ${px(at - GAP_HALF_PX - GAP_FADE_PX)}`,
    `transparent ${px(at - GAP_HALF_PX)}`,
    `transparent ${px(at + GAP_HALF_PX)}`,
    `#000 ${px(at + GAP_HALF_PX + GAP_FADE_PX)}`,
  ]);
  return `linear-gradient(to right, #000, ${stops.join(", ")}, #000)`;
}
