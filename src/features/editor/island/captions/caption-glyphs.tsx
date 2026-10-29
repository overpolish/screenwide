// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CaseSensitive, CaseUpper } from "lucide-react";
import { motion } from "motion/react";
import { type ReactNode, useId, useLayoutEffect, useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { IconSwap } from "../../../../primitives/icon-swap/icon-swap";
import { useGlyphMotion } from "../../../annotations/options/glyph-motion";

import type { TextLines } from "../../../../bindings/TextLines";

// The captions' choices drawn in Lucide's hand (a 24 unit box, a stroke of
// 2, round ends), each filling the box so it reads at an icon's size. What
// a rule leaves out is drawn as itself, a comma, a filler word or a curse,
// and while the rule is on as its Lucide-style off twin, the one swapping
// for the other as the app's on and off icons do.

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

/** Lucide's slash for an icon's "off" twin, corner to corner. */
const SLASH = "m2 2 20 20";

/** The room the slash clears either side of itself in what it crosses, as
 * Lucide's off icons break their strokes round it. */
const SLASH_CLEARANCE = 6;

/** What a rule leaves out, drawn as it is or, once the rule is on, as its
 * Lucide-style off twin: crossed by the slash, with a gap cut round the
 * slash. The two swap as the app's other on and off icons do (`IconSwap`,
 * as `Eye` and `EyeOff` do on the Keyboard island). */
function StruckThrough({
  children,
  isOn,
}: {
  children: ReactNode;
  isOn: boolean;
}) {
  return (
    <span className="relative size-icon">
      <IconSwap swapKey={isOn ? "off" : "on"}>
        {isOn ? <OffTwin>{children}</OffTwin> : <GlyphBox>{children}</GlyphBox>}
      </IconSwap>
    </span>
  );
}

function OffTwin({ children }: { children: ReactNode }) {
  const maskId = useId();
  return (
    <GlyphBox>
      <defs>
        <mask id={maskId}>
          <rect fill="white" height={24} width={24} />
          <path d={SLASH} stroke="black" strokeWidth={SLASH_CLEARANCE} />
        </mask>
      </defs>
      <g mask={`url(#${maskId})`}>{children}</g>
      <path d={SLASH} />
    </GlyphBox>
  );
}

/** The most of the box a word may take, in its 24 units. */
const WORD_ROOM = 21;

/** A context to measure a word's ink with, made once. */
let measuring: CanvasRenderingContext2D | null = null;

/** A short word set bold in the island's type, centred on the box by the
 * ink it draws rather than by its font's line, so lower case such as "um"
 * sits in the middle as symbols do; one too wide is scaled down whole,
 * never squeezed. */
function Word({ children }: { children: string }) {
  const groupRef = useRef<SVGGElement>(null);
  const textRef = useRef<SVGTextElement>(null);
  // Measured once laid out and set on the element itself, before paint:
  // the fit is the drawing's alone, with nothing for React to render. The
  // text's own box is its font's line, ascent to descent, so its ink is
  // measured with the same face on a canvas.
  useLayoutEffect(() => {
    const text = textRef.current;
    measuring ??= document.createElement("canvas").getContext("2d");
    if (!text || !measuring) return;
    measuring.font = `700 16px ${getComputedStyle(text).fontFamily}`;
    const ink = measuring.measureText(children);
    const left = -ink.actualBoundingBoxLeft;
    const width = ink.actualBoundingBoxRight - left;
    const top = -ink.actualBoundingBoxAscent;
    const height = ink.actualBoundingBoxDescent - top;
    if (width === 0) return;
    const scale = Math.min(1, WORD_ROOM / width);
    const x = 12 - (left + width / 2) * scale;
    const y = 12 - (top + height / 2) * scale;
    groupRef.current?.setAttribute(
      "transform",
      `translate(${String(x)} ${String(y)}) scale(${String(scale)})`,
    );
  }, [children]);
  return (
    <g ref={groupRef}>
      <text
        fill="currentColor"
        fontSize={16}
        fontWeight={700}
        ref={textRef}
        stroke="none"
      >
        {children}
      </text>
    </g>
  );
}

/** Two captions as solid blocks one after the other, with the silence
 * between them as a gap; turned on, the first holds until the next starts,
 * closing it. */
export function HoldGlyph({ isOn }: { isOn: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <GlyphBox>
      <motion.rect
        animate={{ width: isOn ? 12 : 6 }}
        fill="currentColor"
        height={14}
        initial={false}
        rx={2.5}
        stroke="none"
        transition={motionOf.move}
        x={1}
        y={5}
      />
      <rect
        fill="currentColor"
        height={14}
        rx={2.5}
        stroke="none"
        width={10}
        x={13}
        y={5}
      />
    </GlyphBox>
  );
}

/** A comma as one shape, its round head running into its tail; turned
 * on, it is struck through. */
export function PunctuationGlyph({ isOn }: { isOn: boolean }) {
  return (
    <StruckThrough isOn={isOn}>
      <path
        d="M12 4a4 4 0 0 1 4 4.5c0 4.8-2.5 8.7-6.6 11.3a1 1 0 0 1-1.2-1.6c2-1.7 3.2-3.7 3.5-6.3A4 4 0 0 1 12 4Z"
        fill="currentColor"
        stroke="none"
      />
    </StruckThrough>
  );
}

/** The language's own filler, such as "um"; turned on, it is struck
 * through. */
export function FillerGlyph({ isOn }: { isOn: boolean }) {
  return (
    <StruckThrough isOn={isOn}>
      <Word>{t("editor-island-captions-filler-sample")}</Word>
    </StruckThrough>
  );
}

/** A curse as comics print one, a run of symbols; turned on, the captions
 * star profanity out, and it is struck through. */
export function ProfanityGlyph({ isOn }: { isOn: boolean }) {
  return (
    <StruckThrough isOn={isOn}>
      <Word>#@!</Word>
    </StruckThrough>
  );
}

/** Lucide's mixed and upper case letters, the one blurring into the other
 * where it sits. */
export function AllCapsGlyph({ isOn }: { isOn: boolean }) {
  return (
    <span className="relative size-icon">
      <IconSwap swapKey={isOn ? "upper" : "mixed"}>
        {isOn ? (
          <CaseUpper aria-hidden className="size-icon" />
        ) : (
          <CaseSensitive aria-hidden className="size-icon" />
        )}
      </IconSwap>
    </span>
  );
}

/** A caption's lines as bars: one, or two. */
export function LinesGlyph({ lines }: { lines: TextLines }) {
  return (
    <GlyphBox>
      {lines === "one" ? <path d="M4 12h16" /> : <path d="M4 9h16M7 15h10" />}
    </GlyphBox>
  );
}
