// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { type ReactNode, useId } from "react";

import { useGlyphMotion } from "../../../annotations/options/glyph-motion";

/** Lucide's pointer, the Cursor tool's own glyph, its tip near the box's top
 * left. */
const POINTER =
  "M4.037 4.688a.495.495 0 0 1 .651-.651l16 6.5a.5.5 0 0 1-.063.947l-6.124 1.58a2 2 0 0 0-1.438 1.435l-1.579 6.126a.5.5 0 0 1-.947.063z";

/** The pointer at `scale`, its tip at `x`, `y`. */
function Pointer({ scale, x, y }: { scale: number; x: number; y: number }) {
  return (
    <path
      d={POINTER}
      transform={`translate(${String(x)} ${String(y)}) scale(${String(scale)}) translate(-4.3 -4.3)`}
    />
  );
}

function GlyphBox({ children }: { children: ReactNode }) {
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      {children}
    </svg>
  );
}

/** The pointer alone, `side` px square, as the sizes draw it. */
export function PointerSizeGlyph({ side }: { side: number }) {
  return (
    <svg
      aria-hidden
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2.5}
      style={{ height: side, width: side }}
      viewBox="0 0 24 24"
    >
      <path d={POINTER} />
    </svg>
  );
}

/** The path the pointer travels, through eleven points along one wave:
 * `shake` px off it, alternately to each side, as raw movement wavers;
 * none, the wave itself. Both are the same points in the same order, so
 * one eases into the other. */
const travel = (shake: number) =>
  Array.from({ length: 11 }, (_, index) => {
    const x = 3 + index * 1.8;
    const wave = 12 - 5 * Math.sin((index / 10) * 2 * Math.PI);
    const off = index === 0 || index === 10 ? 0 : shake * (index % 2 ? 1 : -1);
    return `${index ? "L" : "M"}${x.toFixed(2)} ${(wave + off).toFixed(2)}`;
  }).join("");
const SHAKY = travel(3);
const SMOOTH = travel(0);

/** The path the pointer travels: shaky, as it was recorded; turned on, the
 * shake settles into one smooth wave. */
export function SmoothGlyph({ isSmooth }: { isSmooth: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <motion.path
        animate={{ d: isSmooth ? SMOOTH : SHAKY }}
        initial={false}
        transition={motionOf.move}
      />
    </GlyphBox>
  );
}

/** Where the pointer was a moment before, fainter the further back, as
 * motion blur leaves it: behind it along the way it is going. */
const TRAIL = [
  { at: 4.5, strength: 0.45 },
  { at: 9, strength: 0.2 },
];

/** How far the pointer sits off the middle of the box, to make room behind
 * it for its trail. */
const AHEAD = 4.5;

/** The pointer alone, in the middle; turned on, it moves ahead as fading
 * copies of it slide out behind it, where it has just been. */
export function MotionBlurGlyph({ isBlurred }: { isBlurred: boolean }) {
  const motionOf = useGlyphMotion();
  const lead = isBlurred ? 0 : -AHEAD;
  return (
    <GlyphBox>
      {TRAIL.map(({ at, strength }, index) => (
        <motion.g
          animate={
            isBlurred ? { opacity: strength, x: -at } : { opacity: 0, x: lead }
          }
          initial={false}
          key={at}
          transition={{
            ...motionOf.move,
            delay: isBlurred && motionOf.draw.duration !== 0 ? index * 0.05 : 0,
          }}
        >
          <Pointer scale={0.7} x={11} y={6} />
        </motion.g>
      ))}
      <motion.g
        animate={{ x: lead }}
        initial={false}
        transition={motionOf.move}
      >
        <Pointer scale={0.7} x={11} y={6} />
      </motion.g>
    </GlyphBox>
  );
}

/** The pointer alone; turned on, a ring ripples out from its tip, as a
 * click is drawn. */
export function ClickGlyph({ isAnimated }: { isAnimated: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <motion.circle
        animate={
          isAnimated ? { opacity: 0.6, scale: 1 } : { opacity: 0, scale: 0.2 }
        }
        cx={9}
        cy={9}
        initial={false}
        r={6}
        style={{ originX: 0.5, originY: 0.5 }}
        transition={motionOf.move}
      />
      <Pointer scale={0.72} x={9} y={9} />
    </GlyphBox>
  );
}

/** The edge of the recording, and the pointer half past it: spilling over
 * it, or, turned on, cut off at it, the part outside folding away. */
export function KeepInsideGlyph({ isInside }: { isInside: boolean }) {
  const motionOf = useGlyphMotion();
  const clipId = useId();
  return (
    <GlyphBox>
      <defs>
        <clipPath id={clipId}>
          <motion.rect
            animate={{ width: isInside ? 15 : 24 }}
            height={24}
            initial={false}
            transition={motionOf.move}
            x={0}
            y={0}
          />
        </clipPath>
      </defs>
      <path d="M15 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10" opacity={0.5} />
      <g clipPath={`url(#${clipId})`}>
        <Pointer scale={0.8} x={7} y={8} />
      </g>
    </GlyphBox>
  );
}
