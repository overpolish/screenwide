// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { baseOf } from "../strip/strip-part";
import { type Span } from "../timeline-scale";

// A stack's lanes, which are its bars' stacking order: lane 0 is at the
// top and drawn in front, each lane below it behind. Only bars that show at
// once have an order that matters, and the lanes keep it: of two that
// overlap, the one in the higher lane is drawn in front. The lanes are
// magnetic: each bar rises as high as that order lets it, so one that
// overlaps nothing above it sits in the top lane, and the lanes stay as few
// as they can be. Moving one only changes its order against the bars it
// overlaps; the rest may rise into the room it leaves.

/** A lane's height, in px: a badge's, with a little room. */
export const BAR_PX = 20;

/** A bar in the stack: what it spans, and its lane. */
export type StackedBar = Span & { id: string; lane: number };

/** Where a moved bar goes: into a lane, or into a new lane opened at
 * `lane`, in front of that lane and behind the one above it. */
export type LaneSlot = { isNew: boolean; lane: number };

/** How far into a lane, as a share of its height from either edge, a bar's
 * middle opens a new lane there instead of joining it. */
const EDGE = 0.25;

const overlaps = (first: Span, second: Span) =>
  first.start < second.end && second.start < first.end;

export const laneCountOf = (bars: StackedBar[]) =>
  Math.max(0, ...bars.map(({ lane }) => lane + 1));

/** Whether `moved`, over its span, fits in `lane` beside the bars already
 * there. */
export const fitsIn = (
  bars: StackedBar[],
  moved: Span & { id: string },
  lane: number,
) =>
  bars.every(
    (other) =>
      other.id === moved.id || other.lane !== lane || !overlaps(other, moved),
  );

/** The slot under a bar's middle at `centre`, in lanes from the top of the
 * lanes as they stand: the middle of a lane it fits in joins it; near a
 * lane's edge, or over a lane it does not fit in, a new lane opens on the
 * nearer side. */
export function slotAt(
  bars: StackedBar[],
  id: string,
  centre: number,
): LaneSlot {
  const count = laneCountOf(bars);
  const lane = Math.floor(centre);
  if (lane < 0) return { isNew: true, lane: 0 };
  if (lane >= count) return { isNew: true, lane: count };
  const into = centre - lane;
  const moved = bars.find((bar) => bar.id === id);
  if (into >= EDGE && into <= 1 - EDGE && moved)
    if (fitsIn(bars, moved, lane)) return { isNew: false, lane };
  return { isNew: true, lane: into < 0.5 ? lane : lane + 1 };
}

/** How far, in lanes, a bar's middle must clear a boundary between two
 * slots before it changes slot: either side of every boundary is a band
 * where the slot it has stays, so a few pixels of wobble never restack. */
const STEADY = 0.2;

const sameSlot = (first: LaneSlot, second: LaneSlot) =>
  first.isNew === second.isNew && first.lane === second.lane;

/** The slot under a bar's middle at `centre`, kept at `previous` while the
 * middle is near a boundary and `previous` still holds there. */
export function steadySlot(
  bars: StackedBar[],
  id: string,
  { centre, previous }: { centre: number; previous: LaneSlot | null },
): LaneSlot {
  const slot = slotAt(bars, id, centre);
  if (!previous) return slot;
  const isClear =
    sameSlot(slotAt(bars, id, centre - STEADY), slot) &&
    sameSlot(slotAt(bars, id, centre + STEADY), slot);
  if (isClear) return slot;
  const moved = bars.find((bar) => bar.id === id);
  const stillHolds =
    previous.isNew ||
    (moved !== undefined && fitsIn(bars, moved, previous.lane));
  return stillHolds ? previous : slot;
}

/** Every bar's lane, risen as high as it can go: just under the lowest of
 * the bars it overlaps that are drawn in front of it. The lanes given set
 * that order; where two that overlap share one, the first given is in
 * front. A bar drawn as parts, either side of a jump, is one: its parts
 * keep one lane, the one its first part is given. */
export function packLanes(bars: StackedBar[]): Record<string, number> {
  const wholes = new Map<string, StackedBar[]>();
  for (const bar of bars) {
    const base = baseOf(bar.id);
    wholes.set(base, [...(wholes.get(base) ?? []), bar]);
  }
  const byLane = [...wholes.values()]
    .map((parts, index) => ({ index, parts }))
    .sort(
      (first, second) =>
        first.parts[0].lane - second.parts[0].lane ||
        first.index - second.index,
    )
    .map(({ parts }) => parts);
  const packed: Record<string, number> = {};
  byLane.forEach((parts, index) => {
    let lane = 0;
    for (const front of byLane.slice(0, index))
      if (front.some((each) => parts.some((part) => overlaps(each, part))))
        lane = Math.max(lane, packed[front[0].id] + 1);
    for (const part of parts) packed[part.id] = lane;
  });
  return packed;
}

/** Every bar's lane once `id` is moved into `slot`, risen as high as each
 * can go; the other parts of what `id` is a part of go with it. */
export function restack(
  bars: StackedBar[],
  id: string,
  slot: LaneSlot,
): Record<string, number> {
  // A new lane opens between the lane above it and the one it opens over,
  // so it is numbered half way between until the lanes are packed.
  return packLanes(
    bars.map((bar) =>
      baseOf(bar.id) !== baseOf(id)
        ? bar
        : { ...bar, lane: slot.isNew ? slot.lane - 0.5 : slot.lane },
    ),
  );
}
