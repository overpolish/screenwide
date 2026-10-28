// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";
import { useLocale } from "react-aria";
import { Button } from "react-aria-components";

import {
  cn,
  elementFocusVisible,
  focusStyles,
} from "../../../../../lib/styling";
import { IslandAside } from "../../../../../primitives/island/island-aside";
import { IslandFill } from "../../../../../primitives/island/island-fill";
import { focusIn, glide } from "../../../../../primitives/island/island-motion";
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

/** The column a picker line opens, at the section's end, reaching past
 * the island's row (`IslandAside`): the island grows sideways rather than
 * taller, and its row stays put. It takes its whole width at once, laid out
 * where it will rest, and the island's black reveals it as it opens round
 * it, and hides it as it closes, on the island's one curve. It takes the
 * height the rest of the section has and scrolls within it, adding none of
 * its own, so the island keeps its height as pickers open, close and swap.
 * One picker's choices give way to another's as the island swaps its parts,
 * the column staying open. `content` is null while no picker is open;
 * `contentKey` names whose choices it holds, and `id` is what the open line
 * names as what it opens. */
export function TextPickerColumn({
  content,
  contentKey,
  id,
}: {
  content: ReactNode;
  contentKey: string;
  id: string;
}) {
  const reduce = useReducedMotion();
  return (
    <IslandAside className="relative flex shrink-0 self-stretch">
      {/* The room the column takes, and no height, so it never makes the
       * island taller; what it holds is laid over it. Gone the moment the
       * column closes, so the island starts closing at once. */}
      {content === null ? null : <div className="ms-separate w-72 shrink-0" />}
      <AnimatePresence initial={false}>
        {content === null ? null : (
          // Kept while the black closes over it, which hides it, rather
          // than going before the black has moved.
          <motion.div
            className="absolute inset-y-0 start-0 ms-separate w-72"
            exit={{ opacity: 0 }}
            id={id}
            key="column"
            transition={{
              duration: reduce ? 0 : glide.duration,
              ease: "easeIn",
            }}
          >
            <AnimatePresence initial={false}>
              <motion.div
                animate={focusIn.animate}
                // Under the step past the row the black's top is its outer
                // edge, so the column keeps the section's outer inset from
                // it, not the short one the section keeps under the row.
                className="absolute inset-x-0 top-[calc(var(--island-section-inset)-var(--spacing-inset-sm))] bottom-0"
                exit={focusIn.exit}
                initial={focusIn.exit}
                key={contentKey}
                transition={reduce ? { duration: 0 } : focusIn.transition}
              >
                {content}
              </motion.div>
            </AnimatePresence>
          </motion.div>
        )}
      </AnimatePresence>
    </IslandAside>
  );
}
