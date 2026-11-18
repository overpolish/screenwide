// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  animate,
  motion,
  type MotionValue,
  useMotionValue,
} from "motion/react";
import {
  type ReactNode,
  type Ref,
  use,
  useCallback,
  useLayoutEffect,
  useRef,
} from "react";

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

/** An element's laid-out width to the fraction of a pixel, before its own
 * transforms: `offsetWidth` rounds, and a half folding away on a curve would
 * leave the balance up to half a pixel off on every frame, so the row
 * trembles against the notch. */
function layoutWidth(element: HTMLElement) {
  return parseFloat(getComputedStyle(element).width) || 0;
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
  const towardsEnd = (layoutWidth(endHalf) - layoutWidth(startHalf)) / 2;
  return getComputedStyle(row).direction === "rtl" ? -towardsEnd : towardsEnd;
}

/** Around a notch: each half only as wide as it needs, the notch's width
 * between them. The halves are measured as they change, and the island told
 * how far to move so that gap stays under the notch. Until the island has
 * moved the row there, the row is held there itself (`placeRow`). */
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
  const { isShown, rowShift, setShift } = use(IslandContext);
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
  // The row is moved by the island to the shift last rendered, and a half
  // that resizes between renders, as a part folding away does on every
  // frame, would leave it off the notch until the next render. Rendering
  // there and then is no cure: Motion steps its animations once more after
  // any render it lays out, by however long that render took, and the half
  // moves on again. So the row holds itself where the shift it measured
  // puts it, written straight to the element, and lets go as the island's
  // row catches up.
  const balanceRef = useRef(0);
  const placeRow = useCallback(() => {
    const row = rowRef.current;
    if (!row) return;
    const ahead = balanceRef.current - rowShift.get();
    row.style.transform = ahead === 0 ? "" : `translateX(${String(ahead)}px)`;
  }, [rowShift]);
  useLayoutEffect(() => rowShift.on("change", placeRow), [placeRow, rowShift]);
  // After every render, before paint, so a half that changes with what it
  // holds is balanced in the same frame it is drawn in.
  useLayoutEffect(() => {
    balanceRef.current = balanceShift(
      rowRef.current,
      startRef.current,
      endRef.current,
    );
    setShift(balanceRef.current);
    placeRow();
  });
  // And whenever a half changes size of its own accord, as text settling or
  // a part folding away: the observer is told after layout and before
  // paint, and the island catches up on its next render.
  useLayoutEffect(() => {
    const row = rowRef.current;
    const startHalf = startRef.current;
    const endHalf = endRef.current;
    const observer = new ResizeObserver(() => {
      balanceRef.current = balanceShift(row, startHalf, endHalf);
      placeRow();
      setShift(balanceRef.current);
    });
    if (startHalf) observer.observe(startHalf);
    if (endHalf) observer.observe(endHalf);
    return () => {
      observer.disconnect();
      setShift(0);
    };
  }, [placeRow, setShift]);
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
