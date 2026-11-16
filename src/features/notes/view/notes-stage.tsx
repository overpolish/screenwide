// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode, useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";

// The projects window with notes opened over it. The notes' page grows out
// of their card, uncovered from the card's bounds to the window's while
// the projects fade back and settle a little behind; Back draws the page in
// again to wherever the card stands then. With reduced motion the two
// simply cross. The projects stay as they were underneath, not taken down,
// so their place and scroll are as they were on the way back.

const WHOLE = "inset(0px 0px 0px 0px round 0px)";

/** Where the card of the project at `file` stands, as a clip of the
 * window; the whole window when it is not in view. */
function cardClip(file: string) {
  const picture = document.querySelector(
    `[data-project-file="${CSS.escape(file)}"] [data-project-picture]`,
  );
  if (!picture) return WHOLE;
  const bounds = picture.getBoundingClientRect();
  const radius = getComputedStyle(picture).borderTopLeftRadius;
  return `inset(${String(bounds.top)}px ${String(window.innerWidth - bounds.right)}px ${String(window.innerHeight - bounds.bottom)}px ${String(bounds.left)}px round ${radius})`;
}

const transition = {
  duration: motionDurations.rearrange,
  ease: motionEasings.out,
};
/** The projects go before the page has grown over them, so the two are
 * not read through each other for long. */
const projectsTransition = {
  duration: motionDurations.travel,
  ease: motionEasings.out,
};

export function NotesStage({
  notes,
  page,
  projects,
}: {
  /** The project whose notes are open, by its manifest; null for none. */
  notes: string | null;
  page: ReactNode;
  projects: ReactNode;
}) {
  const isReduced = useReducedMotion() ?? false;
  // Measured as the notes open and as they close, while the projects are
  // laid out underneath either way.
  const [clip, setClip] = useState(WHOLE);
  const [shown, setShown] = useState<string | null>(null);
  if (notes !== shown) {
    setShown(notes);
    setClip(cardClip(notes ?? shown ?? ""));
  }
  return (
    <div className="relative size-full">
      <motion.div
        animate={
          notes
            ? { opacity: 0, scale: isReduced ? 1 : 0.98 }
            : { opacity: 1, scale: 1 }
        }
        className="absolute inset-0 flex"
        inert={notes !== null}
        initial={false}
        transition={projectsTransition}
      >
        {projects}
      </motion.div>
      <AnimatePresence custom={clip}>
        {notes ? (
          <motion.div
            animate={{ clipPath: WHOLE, opacity: 1 }}
            className="absolute inset-0 flex"
            custom={clip}
            exit="hidden"
            initial="hidden"
            key="notes"
            transition={transition}
            variants={{
              hidden: (from: string) => ({
                clipPath: isReduced ? WHOLE : from,
                opacity: 0,
              }),
            }}
          >
            {page}
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
