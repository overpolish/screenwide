// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  BAR_PX,
  fitsIn,
  laneCountOf,
  type LaneSlot,
  packLanes,
  restack,
  type StackedBar,
  steadySlot,
} from "./stack-lanes";
import { type BarHold } from "./use-stack-bar-drag";

// A bar held on the stack, and where every bar goes if it is let go now.
// Where the held bar lands is measured against the lanes as they stood when
// it was picked up, never against the lanes it is reshaping as it moves, so
// the slots under the pointer stay put; and a slot, once taken, is kept
// until the pointer clearly leaves it (`steadySlot`). Carried near the top
// or bottom of the lanes shown, the bar scrolls them on, faster the nearer
// the edge (`carryScrollSpeed`).

/** How far, in px, from an edge of the lanes shown a carried bar starts to
 * scroll them, inside it, and reaches full speed, past it: half a lane, so
 * the pointer held in the middle of the first or last lane shown keeps
 * still, and the margin past the lanes leaves room to speed up. */
const CARRY_EDGE_PX = BAR_PX / 2;
/** How fast, in px a millisecond, a carried bar scrolls the lanes at its
 * fastest. */
const CARRY_SCROLL_MAX = 0.5;

export type Held = BarHold & {
  /** Every bar's lane when this one was picked up. */
  fromLanes: Record<string, number>;
  id: string;
  /** Where the bar is drawn, in px from the top, while carried up and down
   * with the pointer; null while kept to its lane. */
  liftedTop: number | null;
  /** Where it goes if let go now; null to stay in its own lane. */
  slot: LaneSlot | null;
};

/** The bars with the lanes they had when `held` was picked up, the held
 * one where it is now going in time. */
const frozen = (
  bars: StackedBar[],
  held: Pick<Held, "fromLanes" | "id" | "span">,
) =>
  bars.map((bar) => ({
    ...bar,
    ...(bar.id === held.id ? held.span : null),
    lane: held.fromLanes[bar.id] ?? 0,
  }));

/** The next step of holding the bar `id`, from the step before, if it was
 * the same bar, and the lanes the strip shows now. */
export function holdStep({
  bars,
  current,
  hold,
  id,
  shown,
}: {
  bars: StackedBar[];
  current: Held | null;
  hold: BarHold;
  id: string;
  shown: Record<string, number>;
}): Held {
  const before = current?.id === id ? current : null;
  const fromLanes = before ? before.fromLanes : shown;
  const fromLane = fromLanes[id] ?? 0;
  const base = frozen(bars, { fromLanes, id, span: hold.span });
  const moved = hold.y - hold.startY;
  if (hold.isLifted) {
    const count = Math.max(1, laneCountOf(base));
    const liftedTop = Math.min(
      Math.max(fromLane * BAR_PX + moved, -BAR_PX / 2),
      (count - 0.5) * BAR_PX,
    );
    const slot = steadySlot(base, id, {
      centre: liftedTop / BAR_PX + 0.5,
      // Picked up, it starts in its own lane, so the first few pixels of a
      // drag never take it out.
      previous: before?.slot ?? { isNew: false, lane: fromLane },
    });
    return { ...hold, fromLanes, id, liftedTop, slot };
  }
  // Kept to its lane: moved onto another there, it takes a new lane on the
  // side the pointer is, and keeps it while it still overlaps.
  const fits = fitsIn(base, { ...hold.span, id }, fromLane);
  const slot = fits
    ? null
    : before?.slot?.isNew
      ? before.slot
      : { isNew: true, lane: moved < 0 ? fromLane : fromLane + 1 };
  return { ...hold, fromLanes, id, liftedTop: null, slot };
}

/** Every bar's lane if the held one is let go now. */
export function heldLanes(bars: StackedBar[], held: Held) {
  const base = frozen(bars, held);
  return held.slot ? restack(base, held.id, held.slot) : packLanes(base);
}

/** How fast, in px a millisecond, a carried bar scrolls the lanes with the
 * pointer `y` px down the `height` px of them shown: up, below zero, near
 * their top, down near their bottom, faster the nearer, and fastest a
 * little past either edge; still elsewhere. */
export function carryScrollSpeed(y: number, height: number) {
  const nearness = (from: number) =>
    Math.min(Math.max((CARRY_EDGE_PX - from) / (2 * CARRY_EDGE_PX), 0), 1);
  return CARRY_SCROLL_MAX * (nearness(height - y) - nearness(y));
}
