// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { useId } from "react";

import { useGlyphMotion } from "./glyph-motion";

// How a focus treats what lies around its box, drawn in the text's colour as
// the tool's icon is: a field of dots stands for the screen, and the box sits
// clear in its middle while the ring of dots round it is blurred or dimmed.
// Both looks draw the same field, so the two read as one picture with its
// surroundings treated two ways, and changing crossfades the field from
// blurred to dimmed and back while the box stays put.

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

export function FocusLookGlyph({ dims }: { dims: boolean }) {
  const blurId = useId();
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="currentColor"
      viewBox="0 0 24 24"
    >
      <defs>
        <filter height="200%" id={blurId} width="200%" x="-50%" y="-50%">
          <feGaussianBlur stdDeviation={1} />
        </filter>
      </defs>
      {/* The filter sits on a plain group inside the one that fades: Motion
          takes a `filter` on its own element for a style to animate. */}
      <motion.g
        animate={{ opacity: dims ? 0 : 1 }}
        initial={false}
        transition={motionOf.draw}
      >
        <g filter={`url(#${blurId})`}>
          <Dots />
        </g>
      </motion.g>
      <motion.g
        animate={{ opacity: dims ? 0.3 : 0 }}
        initial={false}
        transition={motionOf.draw}
      >
        <Dots />
      </motion.g>
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
