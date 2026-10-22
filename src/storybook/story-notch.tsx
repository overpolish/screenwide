// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { IslandNotch } from "../primitives/island/island-geometry";

// The notch, for stories of the island: what the island is laid out around
// on a Mac that has one, drawn over it since it is not part of the screen.

/** How far the notch's shoulders flare into the top edge, and its bottom
 * corners' radius, in px: the hardware's look, which the screen does not
 * report. */
const FLARE = 6;
const RADIUS = 10;

/** Square shoulders that flare into the top edge and a rounded bottom, in
 * the island's black, so the island is seen to flow round it. */
export function StoryNotch({ notch }: { notch: IslandNotch }) {
  const { height, width } = notch;
  const full = width + FLARE * 2;
  const path = [
    `M0 0 H${String(full)}`,
    `Q${String(full - FLARE)} 0 ${String(full - FLARE)} ${String(FLARE)}`,
    `V${String(height - RADIUS)}`,
    `Q${String(full - FLARE)} ${String(height)} ${String(full - FLARE - RADIUS)} ${String(height)}`,
    `H${String(FLARE + RADIUS)}`,
    `Q${String(FLARE)} ${String(height)} ${String(FLARE)} ${String(height - RADIUS)}`,
    `V${String(FLARE)}`,
    `Q${String(FLARE)} 0 0 0 Z`,
  ].join(" ");
  return (
    <svg
      aria-hidden
      className="pointer-events-none absolute start-1/2 top-0 z-10 -translate-x-1/2 fill-island rtl:translate-x-1/2"
      height={height}
      viewBox={`0 0 ${String(full)} ${String(height)}`}
      width={full}
    >
      <path d={path} />
    </svg>
  );
}
