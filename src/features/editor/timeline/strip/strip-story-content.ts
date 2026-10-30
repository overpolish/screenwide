// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { type TimelineCaption } from "../caption-lane/caption-line";
import { type TimelineCut } from "../cut-seams/cut-seams";
import { type TimelineScene } from "../scene-rail/scene-clip";
import { keptScene, sceneDrawn } from "../scene-rail/scene-story-fixtures";
import { type Span } from "../timeline-scale";

import { type StripBeat } from "./strip-beat-row";
import {
  captionPart,
  clipped,
  partId,
  splitAt,
  splitSpans,
} from "./strip-split";
import { type StoryLull, type StoryRecording } from "./strip-story-fixtures";
import {
  edgesOf,
  moveStretch,
  type PlayOrder,
  putBack,
  rejoined,
} from "./strip-story-order";

import type { LullPlay } from "../../island/lull/lull-island";

// What the strip's stories edit, kept as the editor keeps it: the beats,
// lulls, cuts, scenes, annotations and captions, all in recorded time, and
// the order the recording plays in. A move splits whatever runs across the
// stretch's new edges, each part after the first taking an id of its own.

/** A lull kept, with the end it was recorded with, so a trim can be put
 * back. */
export type LiveLull = StoryLull & { recordedEnd: number };

type Content = {
  annotations: TimelineAnnotation[];
  beats: StripBeat[];
  captions: TimelineCaption[] | null;
  cuts: TimelineCut[];
  lulls: LiveLull[];
  order: PlayOrder;
  scenes: TimelineScene[];
};

const own = <T extends Span>(item: T) => item;
/** What a part was split from: its id before the part's number. */
const baseId = ({ id }: { id: string }) => id.split(":")[0];
/** Spans with ids, their parts made one again where they play together. */
const rejoinSpans = <T extends Span & { id: string }>(
  items: T[],
  order: PlayOrder,
) =>
  rejoined(items, order, {
    join: (first, second) => ({ ...first, end: second.end }),
    keyOf: baseId,
    spanOf: own,
  });

/** `content` laid out for its order: parts that play together again made
 * one, then everything split where the order jumps. */
function relaid(content: Content): Content {
  const { order } = content;
  return split({
    ...content,
    annotations: rejoinSpans(content.annotations, order),
    beats: rejoined(content.beats, order, {
      join: (first, second) => ({
        ...first,
        span: { end: second.span.end, start: first.span.start },
      }),
      keyOf: baseId,
      spanOf: (each) => each.span,
    }),
    cuts: rejoinSpans(content.cuts, order),
    lulls: rejoined(content.lulls, order, {
      join: (first, second) => ({
        ...first,
        moving: [...(first.moving ?? []), ...(second.moving ?? [])],
        recordedEnd: second.recordedEnd,
        span: { end: second.span.end, start: first.span.start },
      }),
      keyOf: baseId,
      spanOf: (each) => each.span,
    }),
    scenes: rejoinSpans(content.scenes, order),
  });
}

/** `content` with everything split where its order jumps. */
function split(content: Content): Content {
  const edges = edgesOf(content.order);
  return {
    annotations: splitSpans(content.annotations, edges),
    beats: splitAt(content.beats, edges, {
      part: (each, span, index) => ({
        ...each,
        id: partId(each.id, index),
        span,
      }),
      spanOf: (each) => each.span,
    }),
    captions:
      content.captions &&
      splitAt(content.captions, edges, { part: captionPart, spanOf: own }),
    cuts: splitSpans(content.cuts, edges),
    lulls: splitAt(content.lulls, edges, {
      part: (each, span, index) => ({
        ...each,
        id: partId(each.id, index),
        moving: clipped(each.moving, span),
        recordedEnd: span.end === each.span.end ? each.recordedEnd : span.end,
        span,
      }),
      spanOf: (each) => each.span,
    }),
    order: content.order,
    scenes: splitSpans(content.scenes, edges),
  };
}

/** The story's content, opened with `moves` already made. */
export function useStoryContent(
  recording: StoryRecording,
  moves: { before: number | null; span: Span }[],
) {
  const [content, setContent] = useState<Content>(() =>
    moves.reduce<Content>(
      (current, { before, span }) =>
        relaid({ ...current, order: moveStretch(current.order, span, before) }),
      {
        ...recording,
        lulls: recording.lulls.map((lull) => ({
          ...lull,
          recordedEnd: lull.span.end,
        })),
        order: [{ end: recording.duration, start: 0 }],
      },
    ),
  );
  const change = (next: (current: Content) => Partial<Content>) => {
    setContent((current) => ({ ...current, ...next(current) }));
  };
  const rail = { duration: recording.duration, scenes: content.scenes };

  return {
    content,
    /** Draws a scene, and gives it, or null for a drag too short. */
    drawScene: (from: number, to: number) => {
      const drawn = sceneDrawn(rail, from, to);
      if (drawn)
        change(({ scenes }) => ({
          scenes: [...scenes, drawn].sort(
            (first, second) => first.start - second.start,
          ),
        }));
      return drawn;
    },
    /** Moves `spans`, in the order they play, to play together before
     * `before`, or at the end for null. */
    move: (spans: Span[], before: number | null) => {
      setContent((current) =>
        relaid({
          ...current,
          order: spans.reduce(
            (order, span) => moveStretch(order, span, before),
            current.order,
          ),
        }),
      );
    },
    putBack: (at: number) => {
      setContent((current) =>
        relaid({ ...current, order: putBack(current.order, at) }),
      );
    },
    restack: (lanes: Record<string, number>) => {
      change(({ annotations }) => ({
        annotations: annotations.map((each) => ({
          ...each,
          lane: lanes[each.id] ?? each.lane,
        })),
      }));
    },
    /** Puts back the cuts `ids`, and any trim among them. */
    restore: (ids: string[]) => {
      change(({ cuts, lulls }) => ({
        cuts: cuts.filter((cut) => !ids.includes(cut.id)),
        lulls: lulls.map((lull) =>
          ids.includes(`trim-${lull.id}`)
            ? { ...lull, span: { ...lull.span, end: lull.recordedEnd } }
            : lull,
        ),
      }));
    },
    retimeAnnotation: (id: string, start: number, end: number) => {
      change(({ annotations }) => ({
        annotations: annotations.map((each) =>
          each.id === id ? { ...each, end, start } : each,
        ),
      }));
    },
    retimeScene: (id: string, start: number, end: number) => {
      change(() => ({ scenes: keptScene(rail, id, { end, start }) }));
    },
    setPlay: (id: string, play: LullPlay) => {
      change(({ lulls }) => ({
        lulls: lulls.map((lull) => (lull.id === id ? { ...lull, play } : lull)),
      }));
    },
    /** Trims the lull `id` to end at `end`; what is trimmed is cut, with a
     * seam to put it back. */
    trim: (id: string, end: number) => {
      change(({ cuts, lulls }) => {
        const lull = lulls.find((each) => each.id === id);
        if (!lull) return {};
        const to = Math.min(
          Math.max(end, lull.span.start + 0.1),
          lull.recordedEnd,
        );
        return {
          cuts: [
            ...cuts.filter((cut) => cut.id !== `trim-${id}`),
            ...(to < lull.recordedEnd
              ? [{ end: lull.recordedEnd, id: `trim-${id}`, start: to }]
              : []),
          ],
          lulls: lulls.map((each) =>
            each.id === id
              ? {
                  ...each,
                  play: { kind: "keep" },
                  span: { ...each.span, end: to },
                }
              : each,
          ),
        };
      });
    },
    /** Trims the beat `id`'s `edge` to `at`, or brings it back: what is
     * trimmed off each end is one cut there, with a seam to put it back. */
    trimBeat: (id: string, edge: "end" | "start", at: number) => {
      change(({ beats, cuts }) => {
        const beat = beats.find((each) => each.id === id);
        if (!beat) return {};
        const cutId = `trim-${edge}-${id}`;
        const span =
          edge === "start"
            ? { end: at, start: beat.span.start }
            : { end: beat.span.end, start: at };
        return {
          cuts: [
            ...cuts.filter((cut) => cut.id !== cutId),
            ...(span.end > span.start ? [{ ...span, id: cutId }] : []),
          ],
        };
      });
    },
  };
}
