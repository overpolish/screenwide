// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnimatePresence,
  motion,
  type MotionValue,
  type Transition,
} from "motion/react";
import { type ReactNode } from "react";

import { motionDurations } from "../../lib/motion";

import { px } from "./island-geometry";
import { focusIn } from "./island-motion";
import { useSectionPlace } from "./island-section-place";
import { useSectionSettle } from "./island-section-settle";

import type { islandLayout } from "./island-layout";
import type { IslandShapeValues } from "./island-outline-sync";

// What hangs beneath the row: a box only as wide as what it holds, under the
// part of the row it belongs to. It paints nothing; the island's outline is
// drawn round it from `values`, which animate it.
export function IslandSection({
  children,
  isFromMiddle,
  isOpening,
  isShown,
  layout,
  movesSideways,
  onElement,
  onOpened,
  rowWidth,
  sectionX,
  travel,
  values,
}: {
  children: ReactNode;
  /** The island coming out of the notch or going back in, which a section
   * opens out of and folds into the middle with. */
  isFromMiddle: boolean;
  /** Opening from closed, until `onOpened`. */
  isOpening: boolean;
  isShown: boolean;
  layout: ReturnType<typeof islandLayout>;
  // Then what hangs is held against the move (`sectionX`) and put where it
  // will rest at once (`island-section-settle.ts`); otherwise it rides with
  // its box.
  /** Whether the island moves sideways to balance its row round a notch. */
  movesSideways: boolean;
  /** The element holding what hangs, to be measured. */
  onElement: (element: HTMLDivElement | null) => void;
  /** The section has finished growing or folding. */
  onOpened: () => void;
  /** The row's width as it is drawn, moving as it grows or shrinks. */
  rowWidth: MotionValue<number>;
  /** Holds what hangs still while the island moves sideways under the
   * notch. */
  sectionX: MotionValue<number>;
  /** How the box moves to a new size (`useIslandFollow`). */
  travel: Transition;
  values: IslandShapeValues["section"];
}) {
  const {
    held,
    inPlace,
    inset,
    isEndAnchored,
    isOpen,
    rowTarget,
    sectionStart,
    sectionWidth,
  } = layout;
  // Hung from the end, what the section holds keeps to the box's end by
  // layout, with no measuring between, and is held to the box's width as it
  // glides: a part that grows or shrinks in place moves with the black's
  // other side, its rows taking the change from the room before their
  // controls, rather than running past the end until the box catches up or
  // jumping ahead of it. A section the row's width leaves only a sliver
  // beside is merged to the row's width, and its rows then span it, names
  // at the start and controls at the end. The island measures it without
  // the hold (`useSize`'s `isNatural`). Not while opening, folding or held
  // round a notch, which place it themselves; but while it opens it is
  // already stretched across the box and placed from the box's start, so it
  // lies exactly as it will once held and nothing moves at the hand-over.
  const isEndPinned = isEndAnchored && isOpen && !isOpening && !movesSideways;
  const stretch = isEndAnchored && isOpen ? sectionWidth - held.width : 0;
  const {
    foldStart,
    followsRow,
    heldStretch,
    holdOffset,
    isHolding,
    isOpeningInPlace,
    lastPlace,
    openFrom,
  } = useSectionPlace({
    isFromMiddle,
    isOpening,
    layout,
    rowWidth,
    stretch,
    travel,
    values,
  });

  // The section's black is there at once and gone once folded, coming out
  // or going back in as much as opening in place: what it holds fades on
  // its own (`useIslandPresence`), and a black fading apart from the row's
  // showed the row as a hard-edged band over a fainter section.
  const opacityTransition = isOpen
    ? { duration: 0 }
    : { delay: motionDurations.travel, duration: 0 };
  const settle = useSectionSettle({
    boxStart: values.start,
    isActive: movesSideways && isOpen && !isOpening,
    target: sectionStart,
  });

  return (
    <div className="absolute start-1/2 top-0">
      <motion.div
        // Its start and width are moved by `useSectionPlace`.
        animate={{ top: rowTarget.height }}
        className="absolute"
        initial={false}
        style={{
          insetInlineStart: values.start,
          top: values.top,
          width: values.width,
        }}
        transition={travel}
      >
        <motion.div
          animate={{
            height: isOpen ? held.height : 0,
            opacity: isOpen ? 1 : 0,
          }}
          // Clips what it holds, which is revealed as it grows. Clipped
          // rather than hidden, so it is no scroll box: a control focused
          // while the box is still short, such as one pressed as the island
          // comes out, cannot scroll what it holds up out of place.
          className="relative overflow-clip"
          initial={false}
          onAnimationComplete={onOpened}
          style={{ height: values.height, opacity: values.opacity }}
          transition={{ ...travel, opacity: opacityTransition }}
        >
          <motion.div
            animate={
              isEndPinned
                ? {}
                : {
                    insetInlineStart: followsRow
                      ? px(inset - stretch)
                      : !isOpen
                        ? px(
                            (isFromMiddle
                              ? inPlace
                              : lastPlace.inset + lastPlace.start - foldStart) -
                              heldStretch,
                          )
                        : isOpening && isFromMiddle
                          ? [px(inPlace - stretch), px(inset - stretch)]
                          : isOpeningInPlace && openFrom !== null
                            ? [
                                px(inset - stretch + sectionStart - openFrom),
                                px(inset - stretch),
                              ]
                            : px(inset - stretch),
                  }
            }
            className="absolute top-0"
            initial={false}
            style={{
              // Held where it will rest while the box opens or folds in
              // place (`useSectionPlace`), its own place set at once.
              marginInlineStart: isHolding ? holdOffset : settle,
              x: movesSideways ? sectionX : 0,
              // Spanning the box, its content set at the end, so content
              // wider than the box overflows at the start, where the black
              // reveals it. Set at the top rather than stretched down, so
              // its height stays its content's: a stretched item's height
              // counts as fixed, and a scroll area inside sizing itself to
              // it would be cut short.
              ...(isEndPinned
                ? {
                    alignItems: "flex-start",
                    display: "flex",
                    insetInlineEnd: 0,
                    insetInlineStart: 0,
                    justifyContent: "flex-end",
                  }
                : {}),
              // Folding, at the width it was stretched to. Held here, since
              // what it holds leaves with the props it had while pinned, a
              // share of this box rather than a width of its own.
              ...(!isOpen && heldStretch > 0 ? { width: lastPlace.width } : {}),
            }}
            transition={isHolding ? { duration: 0 } : travel}
          >
            <AnimatePresence initial={false}>
              {isShown && children ? (
                <motion.div
                  key="section"
                  {...focusIn}
                  className="relative flex w-max shrink-0 flex-col gap-inline px-(--island-section-inset) pt-inset-sm pb-(--island-section-inset)"
                  initial={false}
                  ref={onElement}
                  style={
                    isEndPinned
                      ? { maxWidth: "100%", minWidth: "100%" }
                      : stretch > 0
                        ? { minWidth: px(sectionWidth) }
                        : undefined
                  }
                >
                  {children}
                </motion.div>
              ) : null}
            </AnimatePresence>
          </motion.div>
        </motion.div>
      </motion.div>
    </div>
  );
}
