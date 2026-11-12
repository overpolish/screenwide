// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Lens } from "./fisheye";

// A sound's levels drawn through the fisheye (`fisheye.ts`): a bar every
// few pixels, each as loud as the stretch of sound the lens lays under it,
// so the bars stay evenly spaced while the swell shows the sound in detail
// and the sides show it in summary.

/** How far apart, in px, the bars are drawn. */
const BAR_SPACING_PX = 3;
/** The box the bars are drawn in, stretched to the track's height. */
const HEIGHT = 40;

/** The bars' path: each the mean of the levels under it, as an upright
 * stroke about the middle line, quiet ones keeping a dot. */
function barsPath(
  lens: Lens,
  {
    levels,
    soundMs,
    width,
  }: { levels: number[]; soundMs: number; width: number },
) {
  if (levels.length === 0 || soundMs <= 0) return "";
  const middle = HEIGHT / 2;
  const index = (ms: number) =>
    Math.min(Math.floor((ms / soundMs) * levels.length), levels.length - 1);
  const bars = Math.max(1, Math.floor(width / BAR_SPACING_PX));
  return Array.from({ length: bars }, (_, bar) => {
    const from = index(lens.toMs(bar * BAR_SPACING_PX));
    const to = Math.max(index(lens.toMs((bar + 1) * BAR_SPACING_PX)), from + 1);
    const under = levels.slice(from, to);
    const level = under.reduce((sum, each) => sum + each, 0) / under.length;
    const reach = Math.max(1.25, Math.pow(level, 0.55) * (middle - 2));
    const x = (bar + 0.5) * BAR_SPACING_PX;
    return `M${x.toFixed(2)} ${(middle - reach).toFixed(2)}V${(middle + reach).toFixed(2)}`;
  }).join(" ");
}

export function FisheyeSound({
  label,
  lens,
  levels,
  soundMs,
  width,
}: {
  label: string;
  lens: Lens;
  /** How loud the sound is, from 0 to 1, evenly along it. */
  levels: number[];
  soundMs: number;
  /** The track's width, in px. */
  width: number;
}) {
  return (
    <svg
      aria-label={label}
      className="size-full"
      fill="none"
      preserveAspectRatio="none"
      role="img"
      strokeLinecap="round"
      strokeWidth="2"
      viewBox={`0 0 ${String(Math.max(width, 1))} ${String(HEIGHT)}`}
    >
      <path
        className="stroke-current"
        d={barsPath(lens, { levels, soundMs, width })}
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}
