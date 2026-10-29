// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, type MotionValue } from "motion/react";
import { type ReactNode } from "react";

import { motionDurations } from "../../lib/motion";

import { IslandSectionContext } from "./island-context";
import { px } from "./island-geometry";
import { focusIn, glide } from "./island-motion";
import { useSectionPlace } from "./island-section-place";
import { useSectionSettle } from "./island-section-settle";
import { useSwapHold } from "./island-swap-hold";

import type { islandLayout } from "./island-layout";
import type { IslandShapeValues } from "./island-outline-sync";

/** What hangs beneath the island's row: a box only as wide as what it
 * holds, under the part of the row it belongs to. The box paints nothing;
 * the island's outline is drawn round it, from `values`, which it is
 * animated by. */
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
  /** Whether the island moves sideways to balance its row round a notch.
   * Then what hangs is held against the move (`sectionX`) and put where it
   * will rest at once (`island-section-settle.ts`); otherwise it rides with
   * its box, easing along, and parts swapped are held in place
   * (`island-swap-hold.ts`). */
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
    values,
  });

  // In place, the section is there at once and gone once folded; out of the
  // middle it fades.
  const opacityTransition = isFromMiddle
    ? isOpen
      ? { duration: motionDurations.state }
      : { delay: motionDurations.state, duration: motionDurations.state }
    : isOpen
      ? { duration: 0 }
      : { delay: motionDurations.travel, duration: 0 };
  const swapHold = useSwapHold({
    boxStart: values.start,
    isEnabled: !movesSideways,
    isOpen,
    isOpening,
    target: sectionStart,
  });
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
        transition={glide}
      >
        <motion.div
          animate={{
            height: isOpen ? held.height : 0,
            opacity: isOpen ? 1 : 0,
          }}
          // Clips what it holds, which is revealed as it grows.
          className="relative overflow-hidden"
          initial={false}
          onAnimationComplete={onOpened}
          style={{ height: values.height, opacity: values.opacity }}
          transition={{ ...glide, opacity: opacityTransition }}
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
            transition={isHolding ? { duration: 0 } : glide}
          >
            <IslandSectionContext value={swapHold}>
              <AnimatePresence initial={false}>
                {isShown && children ? (
                  <motion.div
                    key="section"
                    {...focusIn}
                    // Positioned, so a part swapped out is lifted from the
                    // flow (`popLayout`) and only the incoming one is
                    // measured.
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
            </IslandSectionContext>
          </motion.div>
        </motion.div>
      </motion.div>
    </div>
  );
}
