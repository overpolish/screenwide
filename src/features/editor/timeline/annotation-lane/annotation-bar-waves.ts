// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span, type TimelineScale } from "../timeline-scale";

/** Where an annotation's line runs wavy, in px from the line's start. */
export type BarWave = { from: number; to: number };

/** How far a wave rises and falls from the line at most, in px. */
const WAVE_AMPLITUDE_PX = 6;
/** Points along a wave; enough that a long one stays smooth. */
const WAVE_STEPS = 48;
/** The height of `sin 2πt − ½ sin 4πt` at its peak, to scale it to 1. */
const WAVE_PEAK = 1.299;
/** The narrowest a wave is drawn, in px, so a quick move still reads. */
const MIN_WAVE_PX = 24;

/** Each of `places` along the line of a bar for `span`, in px from the
 * line's start. */
export function placesAlong(
  places: Span[],
  { scale, span, width }: { scale: TimelineScale; span: Span; width: number },
): BarWave[] {
  const along = (seconds: number, isEnd?: boolean) =>
    (scale.toX(seconds, isEnd) - scale.toX(span.start)) * width;
  return places
    .map(({ end, start }) => ({ from: along(start), to: along(end, true) }))
    .sort((a, b) => a.from - b.from);
}

/** A line `length` px long at `mid` px down, straight but for one wave over
 * each of `waves`: a peak and a trough that leave and rejoin the line level,
 * so the line runs through them without a corner. A wave too short to read
 * widens about its middle, and none starts before the one before it ends. */
export function wavyLinePath(length: number, waves: BarWave[], mid: number) {
  const point = (x: number, y: number) => `${x.toFixed(2)} ${y.toFixed(2)}`;
  const points = [point(0, mid)];
  let reached = 0;
  for (const wave of waves) {
    const grow = Math.max(0, MIN_WAVE_PX - (wave.to - wave.from)) / 2;
    const from = Math.max(reached, wave.from - grow);
    const to = Math.min(length, Math.max(wave.to + grow, from + MIN_WAVE_PX));
    if (to <= from) continue;
    // A short wave stays a curve rather than a spike.
    const amplitude = Math.min(WAVE_AMPLITUDE_PX, (to - from) / 4);
    for (let step = 0; step <= WAVE_STEPS; step += 1) {
      const t = step / WAVE_STEPS;
      const rise =
        (Math.sin(2 * Math.PI * t) - Math.sin(4 * Math.PI * t) / 2) / WAVE_PEAK;
      points.push(point(from + t * (to - from), mid - rise * amplitude));
    }
    reached = to;
  }
  points.push(point(length, mid));
  return `M${points.join(" L")}`;
}
