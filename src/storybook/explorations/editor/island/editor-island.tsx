// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";

import { Island } from "../../../../primitives/island/island";
import { IslandCrossfade } from "../../../../primitives/island/island-crossfade";
import { ISLAND } from "../../../../primitives/island/island-geometry";
import { focusIn, reveal } from "../../../../primitives/island/island-motion";
import { useSize } from "../../../../primitives/island/island-size";
import { type Editor } from "../model/use-editor";

import { contentOf } from "./island-content";
import { type IslandContent } from "./island-parts";
import { type Spot, spotOf, useIslandPlace } from "./use-island-place";

// Whatever is in hand, on one island that opens where it was asked for, so
// the pointer travels least to reach it (`use-island-place.ts`). Choosing
// something else nearby, the next beat or the next tool, slides it over and
// reshapes it on the way; choosing something elsewhere folds it away where
// it was and brings it out at the new place, rather than sending it across
// the window. With nothing in hand it shrinks to a speck and goes.

type Held = { content: IslandContent; spot: Spot };

export function EditorIsland({
  editor,
  workspace,
}: {
  editor: Editor;
  workspace: HTMLElement | null;
}) {
  // Which part of an island is open under its row, by key: a pen's colours,
  // or a group of an overlay's options. Choosing something else closes it.
  const [openPart, setOpenPart] = useState<string | null>(null);
  const content = contentOf(editor, {
    isOpen: (key) => openPart === key,
    toggle: (key) => {
      setOpenPart((current) => (current === key ? null : key));
    },
  });
  const spot = content ? spotOf(content) : null;
  // What the island shows and where. It changes along with what is in hand
  // while that stays in the same kind of place; moving elsewhere, it keeps
  // the old until it has folded away, and is let go once hidden.
  const [held, setHeld] = useState<Held | null>(
    content && spot ? { content, spot } : null,
  );
  const isMoving = Boolean(held && spot && held.spot.kind !== spot.kind);
  if (
    content &&
    spot &&
    !isMoving &&
    (held?.content.key !== content.key || held.spot.selector !== spot.selector)
  )
    setHeld({ content, spot });
  const isShown = content !== null && !isMoving;
  const shown = isShown ? content : held?.content;
  const { frameRef, partsRef, x, y } = useIslandPlace(
    workspace,
    held?.spot ?? null,
    isShown,
  );
  // The row is as wide as the section under it, so its title keeps to the
  // island's start and its actions to its end, however wide the section
  // makes the island, rather than both being centred in it.
  const [sectionElement, setSectionElement] = useState<HTMLDivElement | null>(
    null,
  );
  const sectionWidth = useSize(sectionElement)?.width ?? 0;
  const rowWidth = shown?.section
    ? sectionWidth + (ISLAND.sectionInset - ISLAND.rowInset) * 2
    : 0;

  return (
    // Laid out left to right like the strip and the picture it reads.
    <div className="pointer-events-none absolute inset-0 z-20" dir="ltr">
      <motion.div
        className="absolute start-0 top-0 flex w-0 justify-center"
        ref={frameRef}
        style={{ x, y }}
      >
        <div className="pointer-events-auto">
          <Island
            anchor={shown?.section?.anchor ?? "center"}
            aria-label="Selection"
            isShown={isShown}
            onHidden={() => {
              setHeld(null);
            }}
            onShapeChange={(parts) => {
              partsRef.current = parts;
            }}
            row={
              shown ? (
                <AnimatePresence initial={false} mode="popLayout">
                  <motion.div
                    animate={reveal.animate}
                    exit={focusIn.exit}
                    initial={reveal.initial}
                    key={shown.key}
                    transition={reveal.transition}
                  >
                    <div
                      className="flex h-10 items-center justify-between gap-separate px-1"
                      style={{ minWidth: rowWidth }}
                    >
                      <div className="flex min-w-0 items-center">
                        {shown.title}
                      </div>
                      {shown.actions ? (
                        <div className="flex shrink-0 items-center gap-separate">
                          {shown.actions}
                        </div>
                      ) : null}
                    </div>
                  </motion.div>
                </AnimatePresence>
              ) : null
            }
            section={
              shown?.section ? (
                <AnimatePresence initial={false} mode="popLayout">
                  <IslandCrossfade key={shown.section.key}>
                    <div ref={setSectionElement}>{shown.section.node}</div>
                  </IslandCrossfade>
                </AnimatePresence>
              ) : undefined
            }
          />
        </div>
      </motion.div>
    </div>
  );
}
