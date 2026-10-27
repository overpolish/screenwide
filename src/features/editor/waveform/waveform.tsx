// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId, useMemo } from "react";

import { cn } from "../../../lib/styling";

/** The box the bars are drawn in, stretched to whatever box it is given;
 * the strokes keep their width as it stretches. */
const WIDTH = 1000;
const HEIGHT = 40;

/** Each bar the loudest of the peaks it stands for, as an upright stroke
 * about the middle line; quiet ones keep a dot, so the line stays whole.
 * Loudness is eased so quiet speech still shows. */
function barsPath(peaks: number[], bars: number) {
  if (peaks.length === 0) return "";
  const middle = HEIGHT / 2;
  return Array.from({ length: bars }, (_, bar) => {
    const from = Math.floor((bar / bars) * peaks.length);
    const to = Math.max(
      from + 1,
      Math.floor(((bar + 1) / bars) * peaks.length),
    );
    const peak = Math.min(1, Math.max(0, ...peaks.slice(from, to)));
    const reach = Math.max(1.25, Math.pow(peak, 0.55) * (middle - 2));
    const x = ((bar + 0.5) / bars) * WIDTH;
    return `M${x.toFixed(2)} ${(middle - reach).toFixed(2)}V${(middle + reach).toFixed(2)}`;
  }).join(" ");
}

/** A sound's peaks as spaced bars, filling with `color` from the start as
 * far as `progress` (0 to 1) has played; the rest is drawn faint. */
export function Waveform({
  "aria-label": label,
  bars = 48,
  className,
  color,
  peaks,
  progress,
}: {
  "aria-label": string;
  /** The colour of what has played. */
  color: string;
  /** Peaks from 0 to 1, evenly along the sound. */
  peaks: number[];
  progress: number;
  /** How many bars the peaks are gathered into, so they stay apart. */
  bars?: number;
  className?: string;
}) {
  const clipId = useId();
  const path = useMemo(() => barsPath(peaks, bars), [bars, peaks]);
  return (
    <svg
      aria-label={label}
      className={cn("text-fg-tertiary", className)}
      fill="none"
      preserveAspectRatio="none"
      role="img"
      strokeLinecap="round"
      strokeWidth="2"
      viewBox={`0 0 ${String(WIDTH)} ${String(HEIGHT)}`}
    >
      <clipPath id={clipId}>
        <rect height={HEIGHT} width={progress * WIDTH} x="0" y="0" />
      </clipPath>
      <path
        className="stroke-current"
        d={path}
        vectorEffect="non-scaling-stroke"
      />
      <path
        clipPath={`url(#${clipId})`}
        d={path}
        stroke={color}
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}
