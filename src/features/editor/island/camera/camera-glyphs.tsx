// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { type ReactNode, useId } from "react";

import { useGlyphMotion } from "../../../annotations/options/glyph-motion";

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

/** Lucide's sparkle, the large star of `Sparkles`. */
const STAR =
  "M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0z";

/** The two small sparkles beside the star, each a cross. */
const GLINTS = [
  { at: { x: 20, y: 5 }, delay: 0 },
  { at: { x: 4, y: 18 }, delay: 0.06 },
];

/** A sparkle, smaller and alone; turned on, it grows and two glints come
 * out beside it, one after the other, as the picture brightens. */
export function EnhanceGlyph({ isEnhanced }: { isEnhanced: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <motion.path
        animate={{ scale: isEnhanced ? 1 : 0.8 }}
        d={STAR}
        initial={false}
        style={{ originX: 0.5, originY: 0.5 }}
        transition={motionOf.move}
      />
      {GLINTS.map(({ at, delay }) => (
        <motion.path
          animate={
            isEnhanced ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0 }
          }
          d={`M${String(at.x)} ${String(at.y - 2)}v4M${String(at.x - 2)} ${String(at.y)}h4`}
          initial={false}
          key={at.x}
          style={{ originX: 0.5, originY: 0.5 }}
          transition={{ ...motionOf.move, delay: isEnhanced ? delay : 0 }}
        />
      ))}
    </GlyphBox>
  );
}

/** A person, head and shoulders, in front of the room as a filled box;
 * turned on, the room fades away to a dashed edge, leaving the person on
 * nothing. */
export function RemoveBackgroundGlyph({ isRemoved }: { isRemoved: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <motion.rect
        animate={{ fillOpacity: isRemoved ? 0 : 0.35 }}
        fill="currentColor"
        height={18}
        initial={false}
        rx={3}
        stroke="none"
        transition={motionOf.fade}
        width={18}
        x={3}
        y={3}
      />
      <motion.rect
        animate={{ opacity: isRemoved ? 0.6 : 0 }}
        height={18}
        initial={false}
        rx={3}
        strokeDasharray="2 3"
        strokeWidth={1.5}
        transition={motionOf.fade}
        width={18}
        x={3}
        y={3}
      />
      <circle cx={12} cy={10} r={3} />
      <path d="M6.5 21a5.5 5.5 0 0 1 11 0" />
    </GlyphBox>
  );
}

/** The person every background glyph draws, head and shoulders. */
function Person() {
  return (
    <>
      <circle cx={12} cy={10} r={3} />
      <path d="M6.5 21a5.5 5.5 0 0 1 11 0" />
    </>
  );
}

/** The room's box, as the Remove background glyph draws it. */
const ROOM = { height: 18, rx: 3, width: 18, x: 3, y: 3 };

/** What is behind the person once the room is taken away, drawn as the
 * Remove background glyph leaves it: nothing, a dashed edge round the
 * person; the room blurred, its fill gone soft; or a picture, a framed
 * scene with its sun behind the person. */
export function BackgroundGlyph({
  background,
}: {
  background: "blurred" | "image" | "transparent";
}) {
  const blurId = useId();
  return (
    <GlyphBox>
      {background === "transparent" ? (
        <rect {...ROOM} opacity={0.6} strokeDasharray="2 3" strokeWidth={1.5} />
      ) : null}
      {background === "blurred" ? (
        <>
          <defs>
            <filter height="140%" id={blurId} width="140%" x="-20%" y="-20%">
              <feGaussianBlur stdDeviation={1.4} />
            </filter>
          </defs>
          <rect
            {...ROOM}
            fill="currentColor"
            fillOpacity={0.45}
            filter={`url(#${blurId})`}
            stroke="none"
          />
        </>
      ) : null}
      {background === "image" ? (
        <>
          <rect {...ROOM} />
          <circle cx={17} cy={7} fill="currentColor" r={1.25} stroke="none" />
        </>
      ) : null}
      <Person />
    </GlyphBox>
  );
}
