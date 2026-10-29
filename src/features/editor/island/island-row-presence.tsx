// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import {
  arrive,
  focusIn,
  glide,
  leave,
  reveal,
} from "../../../primitives/island/island-motion";

const still = { duration: 0 };

/** A part of an island's row that comes and goes with something else, such
 * as a control that only means something while another is on. It takes or
 * leaves its room at once, so the row changes width in one move on the
 * island's own curve; easing its width instead would have the row chase it a
 * frame at a time. While the black is still narrower than the row it is the
 * one part that gives way (its neighbours do not shrink), squeezed and
 * clipped about its middle, so what was there keeps to the black's ends and
 * the part opens with the black; it is revealed once there is mostly room
 * for it, as a section's parts are (`IslandFlow`). It blurs away as it
 * leaves, lifted out of the row (`popLayout`) so the row starts closing at
 * once; lifted out, it keeps to the row's end with the actions beside it,
 * rather than drifting across them with the row's start. Parts beside it
 * that come and go too glide into the room it leaves. */
export function IslandRowPresence({
  children,
  isShown,
}: {
  children: ReactNode;
  isShown: boolean;
}) {
  const reduce = useReducedMotion();
  return (
    // `right` is the row's end: Motion mirrors it in a right-to-left row.
    <AnimatePresence anchorX="right" initial={false} mode="popLayout">
      {isShown ? (
        <motion.span
          animate={reveal.animate}
          // Clipped with room for a focus ring round what it holds.
          className="flex min-w-0 justify-center overflow-clip [overflow-clip-margin:3px] *:shrink-0"
          exit={{ ...focusIn.exit, transition: reduce ? still : leave }}
          initial={reveal.initial}
          key="part"
          layout="position"
          // Inline, over the row's `*:shrink-0`.
          style={{ flexShrink: 1 }}
          transition={reduce ? still : { ...arrive, layout: glide }}
        >
          {children}
        </motion.span>
      ) : null}
    </AnimatePresence>
  );
}
