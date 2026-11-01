// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { type TimelineCaption } from "../caption-lane/caption-line";
import { type TimelineCut } from "../cut-seams/cut-seams";
import { type TimelineScene } from "../scene-rail/scene-clip";
import { type TimelineShortcut } from "../shortcut-lane/shortcut-chip";
import { type Span } from "../timeline-scale";

import { type StripBeat, type StripLull } from "./strip-beat-row";
import { partId } from "./strip-part";

// Where the recording no longer plays straight on, whatever runs across the
// place is drawn as parts, one each side, as the editor splits it when a
// move is made. The strip splits what it draws the same way while a move is
// dragged, so letting go changes nothing on screen.

/** `items` split at each of `edges` inside the span `spanOf` reads from
 * one, each part made by `part` from the item and its share. */
export function splitAt<T>(
  items: T[],
  edges: number[],
  {
    part,
    spanOf,
  }: {
    part: (item: T, span: Span, index: number) => T;
    spanOf: (item: T) => Span;
  },
): T[] {
  return items.flatMap((item) => {
    const { end, start } = spanOf(item);
    const inside = [
      ...new Set(edges.filter((at) => at > start && at < end)),
    ].sort((first, second) => first - second);
    if (inside.length === 0) return [item];
    const bounds = [start, ...inside, end];
    return bounds
      .slice(1)
      .map((to, index) => part(item, { end: to, start: bounds[index] }, index));
  });
}

const own = <T extends Span>(item: T) => item;

/** Spans with ids, split at `edges`. */
export const splitSpans = <T extends Span & { id: string }>(
  items: T[],
  edges: number[],
) =>
  splitAt(items, edges, {
    part: (each, span, index) => ({
      ...each,
      ...span,
      id: partId(each.id, index),
    }),
    spanOf: own,
  });

/** `spans` kept to `to`, leaving out any that fall outside it. */
export const clipped = (spans: Span[] | undefined, to: Span) =>
  spans
    ?.map((each) => ({
      end: Math.min(each.end, to.end),
      start: Math.max(each.start, to.start),
    }))
    .filter((each) => each.end > each.start);

/** How short, in seconds, a part split at a jump is a sliver: an item's
 * edge and a jump's this close are one moment, apart only by rounding. */
export const SLIVER_S = 1e-3;

/** A line's share of `span`: the words said in it. A word said where the
 * span starts, give or take a sliver, is said in it: a jump a rounding
 * after the line's start would otherwise leave its first word to the
 * sliver before the jump, which is not drawn, and the line drawn blank. */
export const captionPart = (line: TimelineCaption, span: Span) => {
  const words = line.words.filter(
    ({ at }) =>
      at >= span.start - SLIVER_S &&
      (span.end >= line.end || at < span.end - SLIVER_S),
  );
  return {
    ...line,
    ...span,
    text: words.map(({ word }) => word).join(" "),
    words,
  };
};

type StripParts = {
  annotations: TimelineAnnotation[];
  beats: StripBeat[];
  captions: TimelineCaption[] | null;
  cuts: TimelineCut[];
  lulls: StripLull[];
  scenes: TimelineScene[];
  shortcuts: TimelineShortcut[];
};

/** Everything the strip draws, split at `edges`. */
export function splitParts<T extends StripParts>(parts: T, edges: number[]): T {
  if (edges.length === 0) return parts;
  return {
    ...parts,
    annotations: splitSpans(parts.annotations, edges),
    beats: splitAt(parts.beats, edges, {
      part: (each, span, index) => ({
        ...each,
        id: partId(each.id, index),
        span,
      }),
      spanOf: (each) => each.span,
    }),
    captions:
      parts.captions &&
      splitAt(parts.captions, edges, { part: captionPart, spanOf: own }),
    cuts: splitSpans(parts.cuts, edges),
    lulls: splitAt(parts.lulls, edges, {
      part: (each, span, index) => ({
        ...each,
        id: partId(each.id, index),
        moving: clipped(each.moving, span),
        plays:
          (each.plays * (span.end - span.start)) /
          (each.span.end - each.span.start),
        span,
      }),
      spanOf: (each) => each.span,
    }),
    scenes: splitSpans(parts.scenes, edges),
    shortcuts: splitSpans(parts.shortcuts, edges),
  };
}
