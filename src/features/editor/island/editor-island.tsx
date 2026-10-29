// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode, useMemo, useState } from "react";

import { t } from "../../../i18n/i18n";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { type IslandAnchor } from "../../../primitives/island/island-geometry";
import {
  focusIn,
  leave,
  reveal,
} from "../../../primitives/island/island-motion";
import { useSize } from "../../../primitives/island/island-size";

// The editor's one island, for whatever is in hand: a row saying what it is,
// its actions at the row's end, and its choices beneath. It is one block, an
// inspector: what is beneath always spans the row, a note included, which
// sits at the block's foot; only a column of choices too many to hold
// inline opens beside it (`IslandAside`). Choosing something else changes
// what it holds in place: the row and the line under it clear into the new
// ones and the section crossfades, while the black reshapes round them.

export type EditorIslandContent = {
  /** Which thing is in hand; a new key brings in a new row and section. */
  key: string;
  title: ReactNode;
  actions?: ReactNode;
  /** Which side of the block the section's content keeps to, and so which
   * side a column of choices opens past. */
  anchor?: IslandAnchor;
  /** A second line in the row's black, under the row, which the section
   * hangs from in its place. */
  below?: ReactNode;
  section?: ReactNode;
  /** Changes when the section changes into something else while the same
   * thing stays in hand, so it crossfades as the black reshapes. */
  sectionKey?: string;
};

export type EditorIslandProps = Pick<
  IslandProps,
  "isShown" | "onHidden" | "onShapeChange"
> & {
  content: EditorIslandContent;
};

/** The row and line leaving go quickly, as a section's part does
 * (`IslandCrossfade`), so the two never read as laid over each other while
 * the new ones are revealed a beat after the black begins to move. */
const leaving = { ...focusIn.exit, transition: leave };

export function EditorIsland({ content, ...island }: EditorIslandProps) {
  // The row keeps its own width as sections change beneath it, so its title
  // and actions stay put while the black reshapes round a section; a
  // section is made to fit under it. Only a second line, which is part of
  // the row's black, widens it, so the title keeps to the island's start
  // and the actions to its end rather than both centring over the line.
  // The line, centred under the row, is at least the row's own width too,
  // so the two are always one width and the line starts where the row does.
  const [rowElement, setRowElement] = useState<HTMLDivElement | null>(null);
  const [belowElement, setBelowElement] = useState<HTMLDivElement | null>(null);
  const rowRef = useMemo(() => latest(setRowElement), []);
  const belowRef = useMemo(() => latest(setBelowElement), []);
  const rowNaturalWidth = useSize(rowElement, { isNatural: true })?.width ?? 0;
  const belowWidth = useSize(belowElement)?.width ?? 0;
  const rowWidth = content.below ? belowWidth : 0;
  // A line arriving under a row that had none is revealed as a new row is;
  // one there from the start is laid out with the island as it comes out.
  const hasBelow = Boolean(content.below);
  const [hadBelow, setHadBelow] = useState(hasBelow);
  const [belowArrives, setBelowArrives] = useState(false);
  if (hadBelow !== hasBelow) {
    setHadBelow(hasBelow);
    setBelowArrives(hasBelow);
  }
  return (
    <Island
      {...island}
      anchor={content.anchor}
      aria-label={t("editor-island-label")}
      below={
        content.below ? (
          <AnimatePresence initial={belowArrives} mode="popLayout">
            <motion.div
              animate={reveal.animate}
              exit={leaving}
              initial={reveal.initial}
              key={content.key}
              ref={belowRef}
              style={{ minWidth: rowNaturalWidth }}
              transition={reveal.transition}
            >
              {content.below}
            </motion.div>
          </AnimatePresence>
        ) : undefined
      }
      row={
        <AnimatePresence initial={false} mode="popLayout">
          <motion.div
            animate={reveal.animate}
            className="min-w-0 grow"
            exit={leaving}
            initial={reveal.initial}
            key={content.key}
            transition={reveal.transition}
          >
            <div
              className="flex h-10 w-max max-w-full items-center justify-between gap-separate px-1"
              ref={rowRef}
              // Across the island where it is wider, and at least the line
              // under it; both left out when its own width is read, which is
              // then its content's (`w-max`). Held to the black as a part
              // arrives, only that part gives way (`IslandRowPresence`).
              style={{ minWidth: `max(${String(rowWidth)}px, 100%)` }}
            >
              <div className="flex shrink-0 items-center">{content.title}</div>
              {content.actions ? (
                <div className="flex min-w-0 items-center gap-separate *:shrink-0">
                  {content.actions}
                </div>
              ) : null}
            </div>
          </motion.div>
        </AnimatePresence>
      }
      // Its title keeps to the island's start and its actions to its end
      // when what hangs under the row is wider than it.
      rowFillsWidth
      section={
        content.section ? (
          <AnimatePresence initial={false} mode="popLayout">
            <IslandCrossfade key={content.sectionKey ?? content.key}>
              <div>{content.section}</div>
            </IslandCrossfade>
          </AnimatePresence>
        ) : undefined
      }
      sectionSpansRow
    />
  );
}

/** A ref for the element of a part that is swapped as what is in hand
 * changes: the part leaving is let go only while it is still the one held,
 * so its going, after the new one has arrived, never leaves none held. */
function latest<T>(set: (update: (current: T | null) => T | null) => void) {
  return (element: T | null) => {
    if (!element) return;
    set(() => element);
    return () => {
      set((current) => (current === element ? null : current));
    };
  };
}
