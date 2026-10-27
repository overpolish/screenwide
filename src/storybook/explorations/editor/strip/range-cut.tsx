// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

import { placed, type StripScale } from "./strip-scale";

// What ⌥ draws on the strip (`use-range-cut.ts`): the blade before a press,
// and the stretch drawn or chosen.

export type Stretch = { end: number; start: number };

/** The stretch being drawn, or the one chosen. */
export function RangeBand({
  scale,
  stretch,
}: {
  scale: StripScale;
  stretch: Stretch;
}) {
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute inset-y-0.5 z-20 rounded-[6px] bg-selected/60 ring-2 ring-primary"
      style={placed(scale, stretch.start, stretch.end)}
    />
  );
}

/** Where a press with ⌥ held will cut, following the pointer. */
export function Blade({ x }: { x: number }) {
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute top-1.5 bottom-0 z-30 w-0"
      style={{ insetInlineStart: `${String(x * 100)}%` }}
    >
      <span className="absolute inset-y-0 -start-px w-0.5 rounded-full bg-primary" />
      <span className="absolute -top-1.5 -start-2.5 flex size-5 items-center justify-center rounded-full bg-primary text-on-primary">
        <Scissors className="size-3" />
      </span>
    </span>
  );
}
