// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { Children, isValidElement, type ReactNode } from "react";

import { cn } from "../../lib/styling";

import { arrive, focusIn, glide, leave, reveal } from "./island-motion";

const still = { duration: 0 };

/** A run of an island's parts, in a line or a column (`className` lays it
 * out), whose parts come and go with choices made elsewhere on the island.
 * The parts that stay glide to their new places on the island's own curve,
 * as the black reshapes round them on it, rather than jumping there while
 * the black is still on its way. One leaving is lifted out of the flow
 * (`popLayout`) and blurs away where it was; one arriving is laid out in its
 * place at once and revealed there once the others have mostly made room,
 * as a section's parts swap (`IslandCrossfade`). A part keeps its identity
 * by its key, or by its place among the children where it has none, so a
 * part given in place of another (`a ? <A /> : <B />`) needs a key of its
 * own to be seen leaving. */
export function IslandFlow({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  const reduce = useReducedMotion();
  // Each part is wrapped to move on its own, keyed as it was given, or by
  // its place among the children.
  // eslint-disable-next-line @eslint-react/no-children-to-array
  const parts = Children.toArray(children).filter(isValidElement);
  return (
    // Positioned, so a part on its way out is lifted out of the flow here.
    <div className={cn("relative", className)}>
      <AnimatePresence initial={false} mode="popLayout">
        {parts.map((part) => (
          <motion.div
            animate={reveal.animate}
            className="flex flex-col"
            exit={{ ...focusIn.exit, transition: reduce ? still : leave }}
            initial={reveal.initial}
            key={part.key}
            layout="position"
            transition={reduce ? still : { ...arrive, layout: glide }}
          >
            {part}
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
