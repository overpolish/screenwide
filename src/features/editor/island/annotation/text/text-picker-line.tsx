// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import { type ReactNode } from "react";
import { useLocale } from "react-aria";
import { Button } from "react-aria-components";

import {
  cn,
  elementFocusVisible,
  focusStyles,
} from "../../../../../lib/styling";
import { IslandFill } from "../../../../../primitives/island/island-fill";
import { useGlyphMotion } from "../../../../annotations/options/glyph-motion";

/** A picker folded to one line across the section: what is chosen, and a
 * chevron that turns to point at the column of choices as it opens beside
 * the section. Pressing it opens them or folds them away again. Quiet at
 * rest, as the island's controls are, filled under the pointer. */
export function TextPickerLine({
  children,
  controls,
  isOpen,
  label,
  onPress,
}: {
  /** What is chosen, as the line shows it. */
  children: ReactNode;
  /** The id of what the line opens. */
  controls: string;
  isOpen: boolean;
  /** The line's accessible name: the picker and what it has chosen. */
  label: string;
  onPress: () => void;
}) {
  const motionOf = useGlyphMotion();
  // The column opens at the section's end, so the chevron turns from
  // pointing down to pointing there: right in a left-to-right language.
  const towardEnd = useLocale().direction === "rtl" ? 90 : -90;
  return (
    <Button
      aria-controls={isOpen ? controls : undefined}
      aria-expanded={isOpen}
      aria-label={label}
      className={cn(
        "group relative isolate flex h-island-control w-full cursor-default items-center justify-between gap-inline rounded-full ps-inset pe-inset-sm text-label text-fg outline-none",
        focusStyles,
        elementFocusVisible,
      )}
      onPress={onPress}
    >
      <IslandFill isChosen={isOpen} />
      <span className="flex min-w-0 items-center gap-inline">{children}</span>
      <motion.span
        animate={{ rotate: isOpen ? towardEnd : 0 }}
        className="flex shrink-0 text-fg-secondary"
        initial={false}
        transition={motionOf.move}
      >
        <ChevronDown aria-hidden className="size-icon" />
      </motion.span>
    </Button>
  );
}
