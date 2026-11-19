// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { AnimatePresence, motion } from "motion/react";
import { type CSSProperties } from "react";

import { motionDurations } from "../../../lib/motion";
import { READING_LINE, readingLineMiddle } from "../reading-line";

/** A cue as it is shown: its picture's address and its project's name. */
export type CardCue = { id: string; src: string; title: string };

/** Where the card stands: filling a window of its own, in the margin the
 * text column leaves at its end (`width` points wide, level with the words
 * of `textSize` being read), or at the top of the text where there is
 * neither. */
export type CardPlace =
  | { kind: "margin"; textSize: number; width: number }
  | { kind: "top" }
  | { kind: "window" };

/** Between the card and what is around it. */
const GAP = "var(--spacing-inset)";

type Styled = { className: string; style?: CSSProperties };

/** The box the card fades in, and the picture's place in it. */
const placed = (place: CardPlace): { box: Styled; picture: Styled } => {
  switch (place.kind) {
    case "window":
      return {
        box: { className: "absolute inset-0" },
        picture: { className: "size-full rounded-panel object-contain" },
      };
    case "margin": {
      // Centred in the margin and level with the words being read, so the
      // cue is a glance to the side of them, and no taller than keeps it
      // inside the frame while level with them.
      const middle = readingLineMiddle(place.textSize);
      return {
        box: {
          className: "pointer-events-none absolute inset-y-0 end-0 z-10",
          style: { width: place.width },
        },
        picture: {
          className:
            "absolute inset-x-0 mx-auto rounded-control object-contain shadow-lg",
          style: {
            maxHeight: `calc(2 * min(${middle} - ${GAP}, 100% - ${middle} - ${GAP}))`,
            maxWidth: `calc(100% - 2 * ${GAP})`,
            top: middle,
            translate: "0 -50%",
          },
        },
      };
    }
    case "top":
      return {
        box: {
          className:
            "pointer-events-none absolute inset-0 z-10 flex items-start justify-end p-inset-sm pe-inset",
        },
        picture: {
          className: "max-w-[45%] rounded-control object-contain shadow-lg",
          // The word being read hangs from the reading line, so the card
          // stops a little above it.
          style: {
            maxHeight: `calc(${String(READING_LINE * 100)}% - 0.75rem)`,
          },
        },
      };
  }
};

// The cue being read towards: in a window of its own beside the
// teleprompter, in the margin beside the text column, or, where there is
// room for neither, at the top of the text's frame, over the lines already
// read and clear of the line being read. It fades in as its word comes near
// and out once its paragraph has been read.
export function CueCard({
  cue,
  onGone,
  place,
}: {
  cue: CardCue | null;
  place: CardPlace;
  /** The cue has faded out with none after it. */
  onGone?: () => void;
}) {
  const { box, picture } = placed(place);
  return (
    <AnimatePresence
      onExitComplete={() => {
        if (!cue) onGone?.();
      }}
    >
      {cue ? (
        <motion.div
          animate={{ opacity: 1 }}
          className={box.className}
          exit={{ opacity: 0 }}
          initial={{ opacity: 0 }}
          key={cue.id}
          style={box.style}
          transition={{ duration: motionDurations.rearrange }}
        >
          <img
            alt={cue.title}
            className={picture.className}
            draggable={false}
            src={cue.src}
            style={picture.style}
          />
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
