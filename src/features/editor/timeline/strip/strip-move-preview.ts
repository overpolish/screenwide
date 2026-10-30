// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span, type TimelineScale, type TimeMap } from "../timeline-scale";

import { type DropTarget } from "./strip-drop-targets";

// While stretches are dragged, the strip is laid out as if they already
// played where they would land, so the rest makes room for them as the
// pointer goes and nothing jumps when they are let go. Where they land is
// read from where the stretches in hand are drawn, against the strip with
// them taken out, so the room opening under the pointer never moves the
// place it opens at.

/** Spans in hand as they are drawn now, as shares of the strip, in the
 * order they play. */
const drawnOf = (spans: Span[], scale: TimelineScale) =>
  spans
    .map((span) => ({
      end: scale.toX(span.end, true),
      start: scale.toX(span.start),
    }))
    .sort((first, second) => first.start - second.start);

const lengthOf = (spans: Span[]) =>
  spans.reduce((sum, { end, start }) => sum + end - start, 0);

/** Where the stretches in hand are drawn while dragged, packed together
 * as they will play: their start and length, as shares of the strip. The
 * one taken hold of at `from` stays under the pointer at `x`. */
export function heldPlace({
  from,
  scale,
  spans,
  x,
}: {
  from: number;
  scale: TimelineScale;
  spans: Span[];
  x: number;
}) {
  const drawn = drawnOf(spans, scale);
  const held = Math.max(
    0,
    drawn.findIndex(({ end, start }) => from >= start && from <= end),
  );
  const before = lengthOf(drawn.slice(0, held));
  return {
    length: lengthOf(drawn),
    start: drawn[held].start - before + x - from,
  };
}

/** The place among `targets` the stretches in hand land at, and where on
 * the strip the room for them opens, as a share of it: the place nearest
 * where they are drawn, measured on the strip with them taken out. The
 * place they land at now is kept until another is nearer by `steady`, or
 * by a third of the way between the two where they are closer than that,
 * so the room never flickers between two as the pointer passes a midpoint
 * yet a place beside a narrow lull can still be reached. */
export function landing({
  current,
  scale,
  spans,
  start,
  steady,
  targets,
}: {
  /** Where they land now, if anywhere yet. */
  current: DropTarget | null;
  scale: TimelineScale;
  spans: Span[];
  /** Where the stretches in hand are drawn from, as a share of the strip. */
  start: number;
  /** How much nearer, as a share of the strip, another place must be. */
  steady: number;
  targets: DropTarget[];
}): { slot: number; target: DropTarget } | null {
  const drawn = drawnOf(spans, scale);
  const slotOf = (target: DropTarget) =>
    target.x - lengthOf(drawn.filter(({ end }) => end <= target.x));
  const off = (target: DropTarget) => Math.abs(slotOf(target) - start);
  let best = current;
  for (const target of targets) {
    const margin =
      best === current && best
        ? Math.min(steady, Math.abs(slotOf(target) - slotOf(best)) / 3)
        : 0;
    if (!best || off(target) < off(best) - margin) best = target;
  }
  return best && { slot: slotOf(best), target: best };
}

/** How near, in seconds, two played times are the same place; reached by
 * different sums, the same place differs in its last bits. */
const SAME_S = 1e-6;

/** `map` as it plays with `spans` moved to play at `at`, a place between
 * them in played seconds, as the strip shows it while they are dragged. */
export function movedMap(map: TimeMap, spans: Span[], at: number): TimeMap {
  const moved = spans
    .map((span) => ({
      ...span,
      from: map.toPlayed(span.start),
      to: map.toPlayed(span.end, true),
    }))
    .sort((first, second) => first.from - second.from);
  const length = moved.reduce((sum, { from, to }) => sum + to - from, 0);
  const removedBefore = (played: number) =>
    moved
      .filter(({ to }) => to <= played + SAME_S)
      .reduce((sum, { from, to }) => sum + to - from, 0);
  // Where the moved stretches start once taken out of the rest.
  const landsAt = at - removedBefore(at);
  return {
    ...map,
    toPlayed: (recorded, isEnd = false) => {
      const played = map.toPlayed(recorded, isEnd);
      let ahead = 0;
      for (const span of moved) {
        const isIn = isEnd
          ? recorded > span.start && recorded <= span.end
          : recorded >= span.start && recorded < span.end;
        if (isIn) return landsAt + ahead + played - span.from;
        ahead += span.to - span.from;
      }
      const rest = played - removedBefore(played);
      return (isEnd ? rest > landsAt + SAME_S : rest >= landsAt - SAME_S)
        ? rest + length
        : rest;
    },
  };
}
