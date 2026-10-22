// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, type MotionValue } from "motion/react";
import { type ReactNode, useState } from "react";

import { motionDurations } from "../../lib/motion";

import { px } from "./island-geometry";
import { type Curve, focusIn } from "./island-motion";

import type { islandLayout } from "./island-layout";
import type { IslandShapeValues } from "./island-outline-sync";

/** What hangs beneath the island's row: a box only as wide as what it
 * holds, under the part of the row it belongs to. The box paints nothing;
 * the island's outline is drawn round it, from `values`, which it is
 * animated by. */
export function IslandSection({
  children,
  curve,
  isFromMiddle,
  isOpening,
  isShown,
  layout,
  onElement,
  onOpened,
  sectionX,
  values,
}: {
  children: ReactNode;
  curve: Curve;
  /** The island coming out of the notch or going back in, which a section
   * opens out of and folds into the middle with. */
  isFromMiddle: boolean;
  /** Opening from closed, until `onOpened`. */
  isOpening: boolean;
  isShown: boolean;
  layout: ReturnType<typeof islandLayout>;
  /** The element holding what hangs, to be measured. */
  onElement: (element: HTMLDivElement | null) => void;
  /** The section has finished growing or folding. */
  onOpened: () => void;
  /** Holds what hangs still while the island moves sideways under the
   * notch. */
  sectionX: MotionValue<number>;
  values: IslandShapeValues["section"];
}) {
  const {
    closed,
    held,
    inPlace,
    inset,
    isOpen,
    rowTarget,
    sectionStart,
    sectionWidth,
  } = layout;
  // A section opens out of the middle, and folds back into it, only as the
  // island comes out of the notch or goes back in. While the island is out,
  // a section opens where it hangs, growing down from under what opened it,
  // and folds up there again; its last place is kept for that.
  const [lastPlace, setLastPlace] = useState({
    inset,
    start: sectionStart,
    width: sectionWidth,
  });
  if (
    isOpen &&
    (lastPlace.inset !== inset ||
      lastPlace.start !== sectionStart ||
      lastPlace.width !== sectionWidth)
  ) {
    setLastPlace({ inset, start: sectionStart, width: sectionWidth });
  }
  // Opening in place, the box and what it holds are put straight where they
  // hang; only its height grows. The outline grows with it, carrying the
  // row's corner down on a side that meets the row's end.
  const isPlacedAtOnce = isOpening && !isFromMiddle;

  // In place, the section is there at once and gone once folded; out of the
  // middle it fades.
  const opacityTransition = isFromMiddle
    ? isOpen
      ? { duration: motionDurations.state }
      : { delay: motionDurations.state, duration: motionDurations.state }
    : isOpen
      ? { duration: 0 }
      : { delay: motionDurations.travel, duration: 0 };

  return (
    <div className="absolute start-1/2 top-0">
      <motion.div
        animate={{
          insetInlineStart: px(
            isOpen
              ? sectionStart
              : isFromMiddle
                ? -closed / 2
                : lastPlace.start,
          ),
          top: rowTarget.height,
          width: isOpen
            ? sectionWidth
            : isFromMiddle
              ? closed
              : lastPlace.width,
        }}
        className="absolute"
        initial={false}
        style={{
          insetInlineStart: values.start,
          top: values.top,
          width: values.width,
        }}
        transition={
          isPlacedAtOnce
            ? {
                ...curve,
                insetInlineStart: { duration: 0 },
                width: { duration: 0 },
              }
            : curve
        }
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
          transition={{ ...curve, opacity: opacityTransition }}
        >
          <motion.div
            animate={{
              insetInlineStart: !isOpen
                ? px(isFromMiddle ? inPlace : lastPlace.inset)
                : isOpening && isFromMiddle
                  ? [px(inPlace), px(inset)]
                  : px(inset),
            }}
            className="absolute top-0"
            initial={false}
            style={{ x: sectionX }}
            transition={
              isPlacedAtOnce
                ? { ...curve, insetInlineStart: { duration: 0 } }
                : curve
            }
          >
            <AnimatePresence initial={false}>
              {isShown && children ? (
                <motion.div
                  key="section"
                  {...focusIn}
                  // Positioned, so a part swapped out is lifted from the
                  // flow (`popLayout`) and only the incoming one is
                  // measured.
                  className="relative flex w-max flex-col gap-inline px-(--island-section-inset) pt-inset-sm pb-(--island-section-inset)"
                  initial={false}
                  ref={onElement}
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
