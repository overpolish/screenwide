// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineCut } from "../cut-seams/cut-seams";
import { type Span, type TimelineScale, type TimeMap } from "../timeline-scale";

// Where moved stretches can land: only between beats, at a cut, or at a
// split, so to put something inside a beat, split or cut there first; or
// back where they came from, which moves nothing.

/** A place a stretch can land: where on the strip, as a share of its width,
 * and in played seconds, and the recorded moment it would then play
 * before, or null for the end; or, staying, where it already plays. */
export type DropTarget = {
  before: number | null;
  played: number;
  x: number;
  isStay?: boolean;
};

/** Every place a stretch can land: where `spans` are now, which keeps them
 * there, then either side of each beat, at each cut and at each split,
 * leaving out those on or inside any of `spans`. */
export function dropTargets({
  beats,
  cuts,
  map,
  scale,
  spans,
  splits,
}: {
  beats: Span[];
  cuts: TimelineCut[];
  map: TimeMap;
  scale: TimelineScale;
  spans: Span[];
  /** The recorded moments the recording is split at. */
  splits: number[];
}): DropTarget[] {
  // What plays straight after the place `played` seconds in.
  const next = (played: number) =>
    played >= map.duration - 1e-6 ? null : map.toRecorded(played);
  const at = (played: number, before: number | null) => ({
    before,
    played,
    x: played / Math.max(map.duration, 1e-6),
  });
  const targets = [
    ...beats.flatMap(({ end, start }) => {
      const after = map.toPlayed(end, true);
      return [at(map.toPlayed(start), start), at(after, next(after))];
    }),
    ...cuts.map(({ start }) => {
      const played = map.toPlayed(start);
      return at(played, next(played));
    }),
    ...splits.map((split) => at(map.toPlayed(split), split)),
  ];
  const isMoved = (x: number) =>
    spans.some(
      ({ end, start }) =>
        x >= scale.toX(start) - 1e-6 && x <= scale.toX(end, true) + 1e-6,
    );
  const first = Math.min(...spans.map(({ start }) => map.toPlayed(start)));
  const stay = { ...at(first, null), isStay: true };
  return [stay, ...targets.filter(({ x }) => !isMoved(x))];
}
