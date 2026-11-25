// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";

import { ShapeGlyph } from "../../glyphs/shape-glyph";

import { useFormatMarkMotion } from "./format-mark-motion";

/** A format's glyph on the format bar, with a dot at its corner while the
 * format leaves out a layer or an annotation the others show. */
export function FormatChipGlyph({
  isLeavingOut = false,
  size,
}: {
  size: { height: number; width: number };
  isLeavingOut?: boolean;
}) {
  const markMotion = useFormatMarkMotion();
  return (
    <span className="relative flex">
      <ShapeGlyph size={size} />
      <AnimatePresence initial={false}>
        {isLeavingOut ? (
          <motion.span
            aria-hidden
            className="absolute -end-0.5 -top-0.5 size-1 rounded-full bg-current"
            key="leaves-out"
            {...markMotion.mark()}
          />
        ) : null}
      </AnimatePresence>
    </span>
  );
}
