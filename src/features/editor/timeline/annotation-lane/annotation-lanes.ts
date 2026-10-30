// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
 * given is in front. */
export function packLanes(annotations: Laned[]): Record<string, number> {
  const byLane = annotations
    .map((annotation, index) => ({ annotation, index }))
    .sort(
      (first, second) =>
        first.annotation.lane - second.annotation.lane ||
        first.index - second.index,
    )
    .map(({ annotation }) => annotation);
  const packed: Record<string, number> = {};
  byLane.forEach((annotation, index) => {
    let lane = 0;
    for (const front of byLane.slice(0, index))
      if (overlaps(front, annotation))
        lane = Math.max(lane, packed[front.id] + 1);
    packed[annotation.id] = lane;
  });
  return packed;
}

/** Every annotation's lane once `id` is moved into `slot`, risen as high
 * as each can go. */
export function restack(
  annotations: Laned[],
  id: string,
  slot: LaneSlot,
): Record<string, number> {
  // A new lane opens between the lane above it and the one it opens over,
  // so it is numbered half way between until the lanes are packed.
  return packLanes(
    annotations.map((annotation) =>
      annotation.id !== id
        ? annotation
        : {
            ...annotation,
            lane: slot.isNew ? slot.lane - 0.5 : slot.lane,
          },
    ),
  );
}

/** The annotations back to front, as the picture draws them: the bottom
 * lane first. Within a lane, as given. */
export const renderOrder = <T extends Laned>(annotations: T[]) =>
  [...annotations].sort((first, second) => second.lane - first.lane);
