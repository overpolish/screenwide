// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode, useState } from "react";

import { t } from "../../../i18n/i18n";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { type IslandAnchor } from "../../../primitives/island/island-geometry";
import { focusIn, reveal } from "../../../primitives/island/island-motion";
import { useSize } from "../../../primitives/island/island-size";

// The editor's one island, for whatever is in hand: a row saying what it is,
// its actions at the row's end, and its choices hanging beneath. Choosing
// something else changes what it holds in place: the row clears into the
// new one and the section crossfades, while the black reshapes round them.

export type EditorIslandContent = {
  /** Which thing is in hand; a new key brings in a new row and section. */
  key: string;
  title: ReactNode;
  actions?: ReactNode;
  /** Where under the row the section hangs. */
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
  const rowNaturalWidth = useSize(rowElement, { isNatural: true })?.width ?? 0;
  const belowWidth = useSize(belowElement)?.width ?? 0;
  const rowWidth = content.below ? belowWidth : 0;
  return (
    <Island
      {...island}
      anchor={content.anchor}
      aria-label={t("editor-island-label")}
      below={
        content.below ? (
          <div
            key={content.key}
            ref={setBelowElement}
            style={{ minWidth: rowNaturalWidth }}
          >
            {content.below}
          </div>
        ) : undefined
      }
      row={
        <AnimatePresence initial={false} mode="popLayout">
          <motion.div
            animate={reveal.animate}
            className="grow"
            exit={focusIn.exit}
            initial={reveal.initial}
            key={content.key}
            transition={reveal.transition}
          >
            <div
              className="flex h-10 w-max items-center justify-between gap-separate px-1"
              ref={setRowElement}
              // Across the island where it is wider, and at least the line
              // under it; both left out when its own width is read, which is
              // then its content's (`w-max`).
              style={{ minWidth: `max(${String(rowWidth)}px, 100%)` }}
            >
              <div className="flex min-w-0 items-center">{content.title}</div>
              {content.actions ? (
                <div className="flex shrink-0 items-center gap-separate">
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
    />
  );
}
