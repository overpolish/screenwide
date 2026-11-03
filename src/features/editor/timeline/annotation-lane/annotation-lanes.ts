// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { baseOf } from "../strip/strip-part";
import { type Span } from "../timeline-scale";

// The annotations' lanes, which are their stacking order: lane 0 is at the
// top and drawn in front, each lane below it behind. Only annotations that
// show at once have an order that matters, and the lanes keep it: of two
// that overlap, the one in the higher lane is drawn in front. The lanes are
// magnetic: each annotation rises as high as that order lets it, so one
// that overlaps nothing above it sits in the top lane, and the lanes stay
// as few as they can be. Moving one only changes its order against the
// annotations it overlaps; the rest may rise into the room it leaves.

type Laned = Span & { id: string; lane: number };

/** Where a moved annotation goes: into a lane, or into a new lane opened
 * at `lane`, in front of that lane and behind the one above it. */
export type LaneSlot = { isNew: boolean; lane: number };

/** How far into a lane, as a share of its height from either edge, a bar's
 * middle opens a new lane there instead of joining it. */
const EDGE = 0.25;

const overlaps = (first: Span, second: Span) =>
  first.start < second.end && second.start < first.end;

export const laneCountOf = (annotations: Laned[]) =>
  Math.max(0, ...annotations.map(({ lane }) => lane + 1));

/** Whether `moved`, over its span, fits in `lane` beside the annotations
 * already there. */
export const fitsIn = (
  annotations: Laned[],
  moved: Span & { id: string },
  lane: number,
) =>
  annotations.every(
    (other) =>
      other.id === moved.id || other.lane !== lane || !overlaps(other, moved),
  );

/** The slot under a bar's middle at `centre`, in lanes from the top of the
 * lanes as they stand: the middle of a lane it fits in joins it; near a
 * lane's edge, or over a lane it does not fit in, a new lane opens on the
 * nearer side. */
export function slotAt(
  annotations: Laned[],
  id: string,
  centre: number,
): LaneSlot {
  const count = laneCountOf(annotations);
  const lane = Math.floor(centre);
  if (lane < 0) return { isNew: true, lane: 0 };
  if (lane >= count) return { isNew: true, lane: count };
  const into = centre - lane;
  const moved = annotations.find((annotation) => annotation.id === id);
  if (into >= EDGE && into <= 1 - EDGE && moved)
    if (fitsIn(annotations, moved, lane)) return { isNew: false, lane };
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
  annotations: Laned[],
  id: string,
  { centre, previous }: { centre: number; previous: LaneSlot | null },
): LaneSlot {
  const slot = slotAt(annotations, id, centre);
  if (!previous) return slot;
  const isClear =
    sameSlot(slotAt(annotations, id, centre - STEADY), slot) &&
    sameSlot(slotAt(annotations, id, centre + STEADY), slot);
  if (isClear) return slot;
  const moved = annotations.find((annotation) => annotation.id === id);
  const stillHolds =
    previous.isNew ||
    (moved !== undefined && fitsIn(annotations, moved, previous.lane));
  return stillHolds ? previous : slot;
}

/** Every annotation's lane, risen as high as it can go: just under the
 * lowest of the annotations it overlaps that are drawn in front of it. The
 * lanes given set that order; where two that overlap share one, the first
 * given is in front. An annotation drawn as parts, either side of a jump,
 * is one: its parts keep one lane, the one its first part is given. */
export function packLanes(annotations: Laned[]): Record<string, number> {
  const clips = new Map<string, Laned[]>();
  for (const annotation of annotations) {
    const base = baseOf(annotation.id);
    clips.set(base, [...(clips.get(base) ?? []), annotation]);
  }
  const byLane = [...clips.values()]
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

/** Every annotation's lane once `id` is moved into `slot`, risen as high
 * as each can go; the other parts of what `id` is a part of go with it. */
export function restack(
  annotations: Laned[],
  id: string,
  slot: LaneSlot,
): Record<string, number> {
  // A new lane opens between the lane above it and the one it opens over,
  // so it is numbered half way between until the lanes are packed.
  return packLanes(
    annotations.map((annotation) =>
      baseOf(annotation.id) !== baseOf(id)
        ? annotation
        : {
            ...annotation,
            lane: slot.isNew ? slot.lane - 0.5 : slot.lane,
          },
    ),
  );
}
