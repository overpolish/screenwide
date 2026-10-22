// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, type MotionValue } from "motion/react";
import { type ReactNode, useState } from "react";

import { motionDurations } from "../../lib/motion";

import { Fillet } from "./island-fillet";
import { islandCurves, px } from "./island-geometry";
import { type Curve, focusIn } from "./island-motion";

import type { islandLayout } from "./island-layout";

/** What hangs beneath the island's row: a box only as wide as what it
 * holds, under the part of the row it belongs to, joined to the row by a
 * curved inner corner on each open side. */
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
  /** The black has finished growing or folding. */
  onOpened: () => void;
  /** Holds what hangs still while the island moves sideways under the
   * notch. */
  sectionX: MotionValue<number>;
}) {
  const {
    closed,
    held,
    inPlace,
    inset,
    isOpen,
    meetsEnd,
    meetsStart,
    reach,
    rowTarget,
    sectionStart,
    sectionWidth,
  } = layout;
  // A section opens out of the middle, and folds back into it, only as the
  // island comes out of the notch or goes back in. While the island is out,
  // a section opens where it hangs, growing down from under what opened it,
  // and folds up there again; its last place is kept for that, with the
  // sides it met the row's end on, which stay square as it folds.
  const [lastPlace, setLastPlace] = useState({
    inset,
    meetsEnd,
    meetsStart,
    start: sectionStart,
    width: sectionWidth,
  });
  if (
    isOpen &&
    (lastPlace.inset !== inset ||
      lastPlace.meetsEnd !== meetsEnd ||
      lastPlace.meetsStart !== meetsStart ||
      lastPlace.start !== sectionStart ||
      lastPlace.width !== sectionWidth)
  ) {
    setLastPlace({
      inset,
      meetsEnd,
      meetsStart,
      start: sectionStart,
      width: sectionWidth,
    });
  }
  const meets = isOpen
    ? { end: meetsEnd, start: meetsStart }
    : { end: lastPlace.meetsEnd, start: lastPlace.meetsStart };
  // Opening in place, the frame and what it holds are put straight where
  // they hang; only the black grows.
  const isPlacedAtOnce = isOpening && !isFromMiddle;

  // The black reaches up behind the row by the row's corner radius. Closed,
  // that reach sits under the row's own rounded corner, the same curve; as
  // the section grows, its side fills in under the row's corner and its own
  // corner carries the curve down, so the row's corner is seen to come down
  // with it, in one motion, and to go back up as it folds. In place it is
  // there at once and gone once folded; out of the middle it fades.
  const curves = islandCurves(reach);
  const black = {
    height: (isOpen ? held.height : 0) + reach,
    opacity: isOpen ? 1 : 0,
  };
  const blackTransition = {
    ...curve,
    opacity: isFromMiddle
      ? isOpen
        ? { duration: motionDurations.state }
        : { delay: motionDurations.state, duration: motionDurations.state }
      : isOpen
        ? { duration: 0 }
        : { delay: motionDurations.travel, duration: 0 },
  };

  // The frame carries where the section hangs and how wide it is, with the
  // fillets attached to its sides; the black inside it grows down. It is
  // drawn behind the row, which its reach runs up into.
  return (
    <div className="absolute start-1/2 top-0 -z-10">
      <motion.div
        animate={{
          insetInlineStart: px(
            isOpen
              ? sectionStart
              : isFromMiddle
                ? -closed / 2
                : lastPlace.start,
          ),
          top: rowTarget.height - reach,
          width: isOpen
            ? sectionWidth
            : isFromMiddle
              ? closed
              : lastPlace.width,
        }}
        className="absolute"
        initial={false}
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
        {/* The black and its joins share one box as tall as the black, a
            size container, so the black's corners and the joins' curves
            can shrink with it together (`islandCurves`). */}
        <motion.div
          animate={black}
          className="relative [container-type:size]"
          initial={false}
          onAnimationComplete={onOpened}
          transition={blackTransition}
        >
          <div
            className="absolute inset-0 overflow-hidden bg-island"
            style={{
              borderEndEndRadius: meets.end ? curves.meeting : curves.open,
              borderEndStartRadius: meets.start ? curves.meeting : curves.open,
            }}
          >
            <motion.div
              animate={{
                insetInlineStart: !isOpen
                  ? px(isFromMiddle ? inPlace : lastPlace.inset)
                  : isOpening && isFromMiddle
                    ? [px(inPlace), px(inset)]
                    : px(inset),
              }}
              className="absolute"
              initial={false}
              style={{ top: reach, x: sectionX }}
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
          </div>
          <Fillet
            isShown={!meets.start}
            radius={curves.fillet}
            side="start"
            top={reach}
          />
          <Fillet
            isShown={!meets.end}
            radius={curves.fillet}
            side="end"
            top={reach}
          />
        </motion.div>
      </motion.div>
    </div>
  );
}
