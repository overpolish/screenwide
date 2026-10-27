// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Still time drawn as what it behaves like: a coil spring, seen from the
// side and a little from above, so each turn is a loop with its far half
// behind its near one. The strip's springs and the spring icon are drawn
// from this one coil, so they are the same shape.

/** How far each turn leans toward the eye, as a share of its height, on
 * the strip: enough to open the loops, not so much the spring looks flat. */
const LEAN = 0.32;
/** Points along each turn: enough for a smooth curve at strip sizes. */
const SAMPLES = 18;

/** A stretch of the wire, on the near or the far half of its turns, as an
 * SVG path. */
export type CoilRun = { d: string; isNear: boolean };

/** The coiled wire across `width`, its axis at `middle`, with `lead` of
 * straight wire before the first turn; `end` is where the coiling stops and
 * the straight wire carries on. `coils` may stop part way through a turn,
 * which ends on the axis at a half turn too, on the far side. */
export function springCoil({
  coils,
  lead,
  lean: leanShare = LEAN,
  middle,
  radius,
  width,
}: {
  coils: number;
  lead: number;
  middle: number;
  radius: number;
  width: number;
  /** The turns' lean, as a share of their height, for a spring taller
   * than the strip's, whose loops would close up at the strip's lean. */
  lean?: number;
}): { end: number; runs: CoilRun[] } {
  const room = width - lead * 2;
  // A turn leans forward by up to twice this, so the coiled part is that
  // much shorter and its loops stay inside the width.
  const lean = Math.min(radius * 2 * leanShare, room / 4);
  const span = room - lean * 2;
  const turns = coils * Math.PI * 2;
  const at = (t: number) => ({
    x: lead + (t / turns) * span + (1 - Math.cos(t)) * lean,
    y: middle - Math.sin(t) * radius,
  });
  const runs: { isNear: boolean; points: string[] }[] = [];
  const total = Math.round(coils * SAMPLES);
  for (let index = 0; index <= total; index += 1) {
    const t = (index / total) * turns;
    // Along the axis, with the turn's lean swinging it forward and back a
    // quarter turn out of step with its rise and fall: that is what draws
    // a loop. Both start and end on the axis, meeting the straight wire.
    const { x, y } = at(t);
    const point = `${x.toFixed(2)} ${y.toFixed(2)}`;
    const isNear = Math.cos(t) >= 0;
    const last = runs[runs.length - 1] as (typeof runs)[number] | undefined;
    if (last?.isNear === isNear) last.points.push(point);
    else {
      // Each run starts where the last ended, so the wire is unbroken.
      const joint = last?.points[last.points.length - 1];
      runs.push({ isNear, points: joint ? [joint, point] : [point] });
    }
  }
  return {
    end: at(turns).x,
    runs: runs.map(({ isNear, points }) => ({
      d: `M${points.join("L")}`,
      isNear,
    })),
  };
}
