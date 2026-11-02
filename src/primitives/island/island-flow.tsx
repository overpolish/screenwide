// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { Children, isValidElement, type ReactNode } from "react";

import { cn } from "../../lib/styling";

import { arrive, focusIn, glide, leave, reveal } from "./island-motion";

const still = { duration: 0 };

// A line or column of island parts (`className` lays it out) that come and go
// with choices made elsewhere. Parts that stay glide on the island's curve as
// the black reshapes; one leaving is lifted out (`popLayout`) and blurs away;
// one arriving is revealed once the others have mostly made room, as
// `IslandCrossfade` swaps. A part keeps its identity by key, or by position
// without one, so `a ? <A /> : <B />` needs keys to be seen leaving.
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
