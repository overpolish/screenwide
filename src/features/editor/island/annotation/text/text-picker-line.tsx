// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import { type ReactNode } from "react";
import { useLocale } from "react-aria";

import { useIslandAsideAtStart } from "../../../../../primitives/island/island-context";
import { useGlyphMotion } from "../../../../annotations/options/glyph-motion";
import { IslandLineButton } from "../../island-line-button";

/** A picker folded to one line across the section: what is chosen, and a
 * chevron that turns to point at the column of choices as it opens beside
 * the section. Pressing it opens them or folds them away again. */
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
  // The chevron turns from pointing down to pointing at the column, on
  // whichever side of the section it opens (`IslandAside`): left for one
  // at the start in a left-to-right language.
  const isLeft = useIslandAsideAtStart() !== (useLocale().direction === "rtl");
  const towardColumn = isLeft ? 90 : -90;
  return (
    <IslandLineButton
      aria-controls={isOpen ? controls : undefined}
      aria-expanded={isOpen}
      aria-label={label}
      className="justify-between"
      isChosen={isOpen}
      onPress={onPress}
    >
      <span className="flex min-w-0 items-center gap-inline">{children}</span>
      <motion.span
        animate={{ rotate: isOpen ? towardColumn : 0 }}
        className="flex shrink-0 text-fg-secondary"
        initial={false}
        transition={motionOf.move}
      >
        <ChevronDown aria-hidden className="size-icon" />
      </motion.span>
    </IslandLineButton>
  );
}
