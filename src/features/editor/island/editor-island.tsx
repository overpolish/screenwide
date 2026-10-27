// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode, useState } from "react";

import { t } from "../../../i18n/i18n";
import { Island, type IslandProps } from "../../../primitives/island/island";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import {
  ISLAND,
  type IslandAnchor,
} from "../../../primitives/island/island-geometry";
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
  // The row is as wide as what is under it, so its title keeps to the
  // island's start and its actions to its end, rather than both being
  // centred over something wider: the second line where there is one,
  // which the section then hangs from at its own width; otherwise the
  // section.
  const [belowElement, setBelowElement] = useState<HTMLDivElement | null>(null);
  const [sectionElement, setSectionElement] = useState<HTMLDivElement | null>(
    null,
  );
  const belowWidth = useSize(belowElement)?.width ?? 0;
  const sectionWidth = useSize(sectionElement)?.width ?? 0;
  const rowWidth = content.below
    ? belowWidth
    : content.section
      ? sectionWidth + (ISLAND.sectionInset - ISLAND.rowInset) * 2
      : 0;
  return (
    <Island
      {...island}
      anchor={content.anchor}
      aria-label={t("editor-island-label")}
      below={
        content.below ? (
          <div key={content.key} ref={setBelowElement}>
            {content.below}
          </div>
        ) : undefined
      }
      row={
        <AnimatePresence initial={false} mode="popLayout">
          <motion.div
            animate={reveal.animate}
            exit={focusIn.exit}
            initial={reveal.initial}
            key={content.key}
            transition={reveal.transition}
          >
            <div
              className="flex h-10 items-center justify-between gap-separate px-1"
              style={{ minWidth: rowWidth }}
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
      section={
        content.section ? (
          <AnimatePresence initial={false} mode="popLayout">
            <IslandCrossfade key={content.sectionKey ?? content.key}>
              <div ref={setSectionElement}>{content.section}</div>
            </IslandCrossfade>
          </AnimatePresence>
        ) : undefined
      }
    />
  );
}
