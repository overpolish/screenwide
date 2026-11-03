// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode, useState } from "react";

import { cn } from "../../lib/styling";
import { glide, leave, reveal } from "../../primitives/island/island-motion";

import { clearIn } from "./settings-motion";

const still = { duration: 0 };

// Room round what it holds inside the clip, so a chosen swatch's outline
// (2 px held 2 px off it) or a focus ring at its edge is not cut while it
// moves. Taken back out by an equal negative margin, so it is no gap.
const RING_ROOM = 4;

/** Something under a row or a heading that comes and goes with a choice made
 * elsewhere, such as a field's picker or a group's note. It opens its height
 * on the island's curve and its content clears out of a blur a beat after,
 * so the space leads and what fills it follows; leaving, the content blurs
 * away quickly while the space closes. Clipped only while it moves, and
 * then with room round what it holds, so a chosen swatch's outline or a
 * focus ring at its edge is never cut. Where motion is reduced it comes and
 * goes at once. A gap above it belongs inside it (`className`), so the gap
 * opens and closes with it. */
export function SettingsReveal({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  const reduce = useReducedMotion() ?? false;
  const isPresent = Boolean(children);
  const [wasPresent, setWasPresent] = useState(isPresent);
  const [isMoving, setIsMoving] = useState(false);
  if (isPresent !== wasPresent) {
    setWasPresent(isPresent);
    setIsMoving(true);
  }
  const settle = () => {
    setIsMoving(false);
  };
  return (
    <AnimatePresence initial={false} onExitComplete={settle}>
      {children ? (
        <motion.div
          animate={{ ...clearIn.animate, height: "auto" }}
          className={cn(isMoving && "overflow-hidden")}
          exit={{
            ...clearIn.exit,
            height: 0,
            overflow: "hidden",
            transition: reduce
              ? still
              : { filter: leave, height: glide, opacity: leave },
          }}
          initial={{ ...clearIn.initial, height: 0 }}
          key="reveal"
          onAnimationComplete={settle}
          style={{ margin: -RING_ROOM, padding: RING_ROOM }}
          transition={reduce ? still : { ...reveal.transition, height: glide }}
        >
          <div className={className}>{children}</div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
