// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Eye, EyeOff } from "lucide-react";
import { motion } from "motion/react";
import { type ReactNode, useId } from "react";

import { IconSwap } from "../../../../primitives/icon-swap/icon-swap";
import { useGlyphMotion } from "../../../annotations/options/glyph-motion";

/** A key's cap, and the label on it, as one shortcut is drawn. */
const CAP = { height: 14, rx: 3.5, width: 14, x: 5, y: 5 };
const LABEL = "M9.5 14.5h5";

function GlyphBox({
  children,
  side,
}: {
  children: ReactNode;
  /** Drawn `side` px square; the icon's size where omitted. */
  side?: number;
}) {
  return (
    <svg
      aria-hidden
      className={side === undefined ? "size-icon" : undefined}
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      style={side === undefined ? undefined : { height: side, width: side }}
      viewBox="0 0 24 24"
    >
      {children}
    </svg>
  );
}

/** A keycap, `side` px square, as the sizes draw it. */
export function KeycapSizeGlyph({ side }: { side: number }) {
  return (
    <GlyphBox side={side}>
      <rect {...CAP} strokeWidth={2.5} />
      <path d={LABEL} strokeWidth={2.5} />
    </GlyphBox>
  );
}

/** Lucide's eye while the shortcuts show, and its eye struck through once
 * they are hidden, the one blurring into the other where it sits. */
export function ShowGlyph({ isShown }: { isShown: boolean }) {
  return (
    <span className="relative size-icon">
      <IconSwap swapKey={isShown ? "shown" : "hidden"}>
        {isShown ? (
          <Eye aria-hidden className="size-icon" />
        ) : (
          <EyeOff aria-hidden className="size-icon" />
        )}
      </IconSwap>
    </span>
  );
}

/** A keycap as the shortcuts are dressed: drawn in outline for dark keys,
 * which the black shows through; filled for light, its label cut out of
 * the fill. */
export function AppearanceGlyph({ isLight }: { isLight: boolean }) {
  const motionOf = useGlyphMotion();
  const maskId = useId();
  return (
    <GlyphBox>
      <defs>
        <mask id={maskId}>
          <rect fill="white" height={24} width={24} />
          <path d={LABEL} stroke="black" />
        </mask>
      </defs>
      <motion.rect
        {...CAP}
        animate={{ fillOpacity: isLight ? 1 : 0 }}
        fill="currentColor"
        initial={false}
        mask={`url(#${maskId})`}
        transition={motionOf.fade}
      />
      <motion.path
        animate={{ opacity: isLight ? 0 : 1 }}
        d={LABEL}
        initial={false}
        transition={motionOf.fade}
      />
    </GlyphBox>
  );
}
