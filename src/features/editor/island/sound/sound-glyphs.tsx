// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { type ReactNode } from "react";

import { useGlyphMotion } from "../../../annotations/options/glyph-motion";

// The microphone's tools, each drawn in Lucide's hand (a 24 unit box, a
// stroke of 2, round ends) with a look for off and one for on that it
// moves between, as the Camera and Cursor islands' toggles do, so a tool
// says whether it is on by its glyph alone.

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

/** Lucide's microphone, a little to the start; turned on, a glint comes
 * out beside its head, as the voice is rebuilt as a studio recording. */
export function StudioSoundGlyph({ isOn }: { isOn: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <path d="M10 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M17 10v2a7 7 0 0 1-14 0v-2" />
      <path d="M10 19v3" />
      <motion.path
        animate={isOn ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0 }}
        d="M20 2v5M17.5 4.5h5"
        initial={false}
        style={{ originX: 0.5, originY: 0.5 }}
        transition={motionOf.move}
      />
    </GlyphBox>
  );
}

/** Specks of noise scattered round the voice's wave; turned on, they fade
 * away, leaving the wave clean. */
const SPECKS = [
  { x: 4, y: 5 },
  { x: 10, y: 4 },
  { x: 19, y: 5 },
  { x: 5, y: 19 },
  { x: 14, y: 20 },
  { x: 20, y: 19 },
];

export function ReduceNoiseGlyph({ isOn }: { isOn: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <path d="M2 12c1.7-3.3 3.3-3.3 5 0s3.3 3.3 5 0 3.3-3.3 5 0 3.3 3.3 5 0" />
      {SPECKS.map(({ x, y }, index) => (
        <motion.path
          animate={{ opacity: isOn ? 0 : 1 }}
          d={`M${String(x)} ${String(y)}h.01`}
          initial={false}
          key={`${String(x)}-${String(y)}`}
          transition={{ ...motionOf.fade, delay: isOn ? index * 0.03 : 0 }}
        />
      ))}
    </GlyphBox>
  );
}

/** The voice as Lucide's audio lines, jagged, with spikes standing out
 * where clicks and harsh "s" sounds do; turned on, the lines ease into one
 * smooth swell. */
const VOICE_LINES = [
  { off: 8, on: 4, x: 3 },
  { off: 3, on: 9, x: 7.5 },
  { off: 20, on: 13, x: 12 },
  { off: 5, on: 9, x: 16.5 },
  { off: 14, on: 4, x: 21 },
];

export function VocalCleanupGlyph({ isOn }: { isOn: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      {VOICE_LINES.map(({ off, on, x }) => {
        const half = (isOn ? on : off) / 2;
        return (
          <motion.line
            animate={{ y1: 12 - half, y2: 12 + half }}
            initial={false}
            key={x}
            transition={motionOf.move}
            x1={x}
            x2={x}
          />
        );
      })}
    </GlyphBox>
  );
}

/** A level meter's bars, rising from one floor to uneven heights under the
 * level they are brought to, drawn faint; turned on, every bar rises or
 * falls to meet it, as the voice is brought to one steady loudness. */
const LEVEL = 7;
const LEVELS = [
  { off: 13, x: 5 },
  { off: 5, x: 10 },
  { off: 16, x: 15 },
  { off: 9, x: 20 },
];

export function AutoVolumeGlyph({ isOn }: { isOn: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <path d="M2 5h20" opacity={0.5} strokeDasharray="2 3" strokeWidth={1.5} />
      {LEVELS.map(({ off, x }) => (
        <motion.line
          animate={{ y1: isOn ? LEVEL : 21 - off }}
          initial={false}
          key={x}
          transition={motionOf.move}
          x1={x}
          x2={x}
          y2={21}
        />
      ))}
    </GlyphBox>
  );
}

/** A note of the system audio's music; turned on, it sinks and shrinks
 * as an arrow over it points down, the singing making way for the voice. */
export function StudioDuckingGlyph({ isOn }: { isOn: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <motion.g
        animate={isOn ? { scale: 0.8, y: 3 } : { scale: 1, y: 0 }}
        initial={false}
        style={{ originX: 0.5, originY: 1 }}
        transition={motionOf.move}
      >
        <path d="M9 18V6l10-2v12" />
        <circle cx={6.5} cy={18} r={2.5} />
        <circle cx={16.5} cy={16} r={2.5} />
      </motion.g>
      <motion.path
        animate={isOn ? { opacity: 1, y: 0 } : { opacity: 0, y: -3 }}
        d="M2 4.5l2.5 2.5L7 4.5"
        initial={false}
        transition={motionOf.move}
      />
    </GlyphBox>
  );
}
