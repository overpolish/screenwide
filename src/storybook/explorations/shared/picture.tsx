// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";

import { waveform } from "./mock-data";

/** Level bars along a lane; the bar colour is the element's text colour. */
export function Waveform({
  bars = 160,
  className,
  seed = 1,
}: {
  bars?: number;
  className?: string;
  seed?: number;
}) {
  return (
    <svg
      aria-hidden
      className={cn("size-full", className)}
      preserveAspectRatio="none"
      viewBox={`0 0 ${String(bars * 2)} 100`}
    >
      {waveform(bars, seed).map((level, index) => (
        <rect
          fill="currentColor"
          height={level * 80}
          // A fixed sample: the bars never reorder.
          // eslint-disable-next-line @eslint-react/no-array-index-key
          key={index}
          rx={0.6}
          width={1.2}
          x={index * 2}
          y={50 - level * 40}
        />
      ))}
    </svg>
  );
}
