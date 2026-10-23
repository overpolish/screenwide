// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId } from "react";

// How a focus treats what lies around its box, drawn in the text's colour as
// the tool's icon is: a field of dots stands for the screen, and the box sits
// clear in its middle while the ring of dots round it is blurred or dimmed.
// Both looks draw the same field, so the pair reads as one picture with its
// surroundings treated two ways.

const STEPS = [4.5, 9.5, 14.5, 19.5];
const INSIDE = new Set([9.5, 14.5]);
/** The dots round the box; the four it would frame are left out. */
const DOTS = STEPS.flatMap((y) => STEPS.map((x) => ({ x, y }))).filter(
  ({ x, y }) => !(INSIDE.has(x) && INSIDE.has(y)),
);

function Dots() {
  return DOTS.map(({ x, y }) => (
    <circle cx={x} cy={y} key={`${String(x)} ${String(y)}`} r={1.4} />
  ));
}

export function FocusLookGlyph({ look }: { look: "blur" | "dim" }) {
  const blurId = useId();
  return (
    <svg
      aria-hidden
      className="size-icon-lg"
      fill="currentColor"
      viewBox="0 0 24 24"
    >
      {look === "blur" ? (
        <>
          <defs>
            <filter height="200%" id={blurId} width="200%" x="-50%" y="-50%">
              <feGaussianBlur stdDeviation={1} />
            </filter>
          </defs>
          <g filter={`url(#${blurId})`}>
            <Dots />
          </g>
        </>
      ) : (
        <g opacity={0.3}>
          <Dots />
        </g>
      )}
      <rect
        fill="none"
        height={10}
        rx={2}
        stroke="currentColor"
        strokeWidth={1.5}
        width={10}
        x={7}
        y={7}
      />
    </svg>
  );
}
