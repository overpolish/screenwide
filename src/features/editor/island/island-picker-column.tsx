// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import { IslandAside } from "../../../primitives/island/island-aside";
import { focusIn, glide } from "../../../primitives/island/island-motion";

/** A column of choices opened beside an island's section, at its end,
 * reaching past the island's row (`IslandAside`): the island grows sideways
 * rather than taller, and its row stays put. It takes its whole width at
 * once, laid out where it will rest, and the island's black reveals it as it
 * opens round it, and hides it as it closes, on the island's one curve. It
 * takes the height the rest of the section has and scrolls within it,
 * adding none of its own, so the island keeps its height as choices open,
 * close and swap. One set of choices gives way to another as the island
 * swaps its parts, the column staying open. `content` is null while nothing
 * is open; `contentKey` names whose choices it holds, and `id` is what
 * opened it names as what it opens. */
export function IslandPickerColumn({
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
