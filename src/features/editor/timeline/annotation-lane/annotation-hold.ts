// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ANNOTATION_BAR_PX } from "./annotation-bar";
import {
  fitsIn,
  laneCountOf,
  type LaneSlot,
  packLanes,
  restack,
  steadySlot,
} from "./annotation-lanes";
import { type BarHold } from "./use-annotation-bar-drag";

import type { TimelineAnnotation } from "./annotation-lane";

// A bar held on the lane, and where every bar goes if it is let go now.
// Where the held bar lands is measured against the lanes as they stood when
// it was picked up, never against the lanes it is reshaping as it moves, so
// the slots under the pointer stay put; and a slot, once taken, is kept
// until the pointer clearly leaves it (`steadySlot`).

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
  annotations: TimelineAnnotation[],
  held: Pick<Held, "fromLanes" | "id" | "span">,
) =>
  annotations.map((annotation) => ({
    ...annotation,
    ...(annotation.id === held.id ? held.span : null),
    lane: held.fromLanes[annotation.id] ?? 0,
  }));

/** The next step of holding the bar `id`, from the step before, if it was
 * the same bar, and the lanes the strip shows now. */
export function holdStep({
  annotations,
  current,
  hold,
  id,
  shown,
}: {
  annotations: TimelineAnnotation[];
  current: Held | null;
  hold: BarHold;
  id: string;
  shown: Record<string, number>;
}): Held {
  const before = current?.id === id ? current : null;
  const fromLanes = before ? before.fromLanes : shown;
  const fromLane = fromLanes[id] ?? 0;
  const base = frozen(annotations, { fromLanes, id, span: hold.span });
  const moved = hold.clientY - hold.startY;
  if (hold.isLifted) {
    const count = Math.max(1, laneCountOf(base));
    const liftedTop = Math.min(
      Math.max(fromLane * ANNOTATION_BAR_PX + moved, -ANNOTATION_BAR_PX / 2),
      (count - 0.5) * ANNOTATION_BAR_PX,
    );
    const slot = steadySlot(base, id, {
      centre: liftedTop / ANNOTATION_BAR_PX + 0.5,
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
export function heldLanes(annotations: TimelineAnnotation[], held: Held) {
  const base = frozen(annotations, held);
  return held.slot ? restack(base, held.id, held.slot) : packLanes(base);
}
