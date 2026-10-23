// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  animate,
  motion,
  type MotionValue,
  useMotionValue,
} from "motion/react";
import { type ReactNode, type Ref, use, useLayoutEffect, useRef } from "react";

import { cn } from "../../lib/styling";

import { IslandContext } from "./island-context";
import { ISLAND, type IslandNotch } from "./island-geometry";
import { glide, reveal } from "./island-motion";

// The island's row: what it holds at its start and its end. Around a notch
// the two sides are parted by the notch's width, each only as wide as it
// needs, and the island is told how far to move so the gap stays under the
// notch; without one they sit side by side.

/** One half of the row, revealed where it sits as the island grows: it comes
 * into focus and to its size, as though the glass were clearing, and never
 * travels in. Around a notch it folds into it as the island tucks away
 * (`x`). */
function Half({
  children,
  notch,
  notchSide,
  ref,
  x,
}: {
  children: ReactNode;
  notch?: IslandNotch;
  /** Which edge of this half meets the notch. */
  notchSide?: "end" | "start";
  ref?: Ref<HTMLSpanElement>;
  x?: MotionValue<number>;
}) {
  const room = notch ? notch.width / 2 + ISLAND.notchGap : 0;
  return (
    <motion.span
      {...reveal}
      className="relative flex items-center gap-separate"
      ref={ref}
      style={{
        x,
        ...(notchSide === "end"
          ? { paddingInlineEnd: room }
          : notchSide === "start"
            ? { paddingInlineStart: room }
            : undefined),
      }}
    >
      {children}
    </motion.span>
  );
}

/** How far the island moves so the gap between the halves, where the notch
 * is, sits under the notch: half the difference, towards the narrower side;
 * the start is on the right in a right-to-left layout. */
function balanceShift(
  row: HTMLElement | null,
  startHalf: HTMLElement | null,
  endHalf: HTMLElement | null,
) {
  if (!row || !startHalf || !endHalf) return 0;
  const towardsEnd = (endHalf.offsetWidth - startHalf.offsetWidth) / 2;
  return getComputedStyle(row).direction === "rtl" ? -towardsEnd : towardsEnd;
}

/** Around a notch: each half only as wide as it needs, the notch's width
 * between them. The halves are measured as they change, and the island told
 * how far to move so that gap stays under the notch. */
function NotchSplit({
  end,
  notch,
  start,
}: {
  end: ReactNode;
  notch: IslandNotch;
  start: ReactNode;
}) {
  const rowRef = useRef<HTMLDivElement>(null);
  const startRef = useRef<HTMLSpanElement>(null);
  const endRef = useRef<HTMLSpanElement>(null);
  const { isShown, setShift } = use(IslandContext);
  const startX = useMotionValue(0);
  const endX = useMotionValue(0);
  // Tucking away, each half folds into the notch with its own edge of the
  // black: as far as what it holds is wide, on the shape's own curve, so a
  // wider half travels further, as its edge does, and neither is left behind
  // as the shape slides under it. Coming back out, both are in place.
  useLayoutEffect(() => {
    const row = rowRef.current;
    const startHalf = startRef.current;
    const endHalf = endRef.current;
    if (!row || !startHalf || !endHalf) return;
    const room = notch.width / 2 + ISLAND.notchGap;
    const towardsEnd = getComputedStyle(row).direction === "rtl" ? -1 : 1;
    const folds = [
      animate(
        startX,
        isShown ? 0 : towardsEnd * (startHalf.offsetWidth - room),
        glide,
      ),
      animate(
        endX,
        isShown ? 0 : -towardsEnd * (endHalf.offsetWidth - room),
        glide,
      ),
    ];
    return () => {
      folds.forEach((fold) => {
        fold.stop();
      });
    };
  }, [endX, isShown, notch.width, startX]);
  // After every render, before paint, so a half that changes with what it
  // holds is balanced in the same frame it is drawn in; an observer would
  // answer a frame late, and that frame would show the island jump.
  useLayoutEffect(() => {
    setShift(balanceShift(rowRef.current, startRef.current, endRef.current));
  });
  // And whenever a half changes size of its own accord, as text settling.
  useLayoutEffect(() => {
    const row = rowRef.current;
    const startHalf = startRef.current;
    const endHalf = endRef.current;
    const observer = new ResizeObserver(() => {
      setShift(balanceShift(row, startHalf, endHalf));
    });
    if (startHalf) observer.observe(startHalf);
    if (endHalf) observer.observe(endHalf);
    return () => {
      observer.disconnect();
      setShift(0);
    };
  }, [setShift]);
  return (
    <div className={ROW} ref={rowRef}>
      <Half notch={notch} notchSide="end" ref={startRef} x={startX}>
        {start}
      </Half>
      <Half notch={notch} notchSide="start" ref={endRef} x={endX}>
        {end}
      </Half>
    </div>
  );
}

/** A control's height and the row's inset around it, top and bottom. */
const ROW = "flex h-10 items-center px-1";

export function IslandRow({
  end,
  start,
}: {
  end?: ReactNode;
  start?: ReactNode;
}) {
  const { notch } = use(IslandContext);
  if (notch) return <NotchSplit end={end} notch={notch} start={start} />;
  // An empty half is left out, not laid out, so a row with one side only
  // has no gap standing in for the other.
  return (
    <div className={cn(ROW, "gap-separate")}>
      {start == null ? null : <Half>{start}</Half>}
      {end == null ? null : <Half>{end}</Half>}
    </div>
  );
}
