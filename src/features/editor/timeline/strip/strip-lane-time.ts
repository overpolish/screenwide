// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import {
  recordedTime,
  type Span,
  timelineScale,
  type TimelineScale,
  type TimeMap,
} from "../timeline-scale";

import { partId } from "./strip-part";
import { SLIVER_S, splitSpans } from "./strip-split";
import { type TimelineStripProps } from "./timeline-strip-props";

// The scenes, the annotations and the songs and sound effects are timed on
// the timeline as it plays, so their lanes lay them out in played seconds:
// a bar is one piece wherever it runs, across a jump or a gap alike, and a
// drag, a drawing or a snap moves it in time as it plays. What belongs to
// the footage (beats, lulls, cuts, captions, shortcuts) keeps recorded
// seconds, so a cut or a squeeze never moves it off what it was read from.
// Only while a move is dragged, which lays the strip out as if let go, are
// the lanes drawn where their footage will play: in recorded seconds, in
// parts either side of each jump, as letting go splits them (`splitParts`).

/** A recorded span where it plays, in played seconds. */
export const playedSpan = (map: TimeMap, { end, start }: Span): Span => ({
  end: map.toPlayed(end, true),
  start: map.toPlayed(start),
});

/** A stretch's span in recorded seconds, from its span in played seconds,
 * inside one run. */
function recordedSpan(map: TimeMap, from: number, to: number): Span {
  const start = map.toRecorded(from);
  // Its end is the last moment it shows, which is in what plays before it.
  const end = map.toRecorded(Math.max(to - 1e-4, from)) + 1e-4;
  return { end: Math.max(end, start), start };
}

/** The parts of a span in played seconds, in recorded seconds: one for
 * each run it plays across, with the run in played seconds it lies in.
 * One span from its start to its end would take in what plays elsewhere
 * between them, or run backwards. */
const recordedParts = (map: TimeMap, runs: Span[], span: Span) =>
  runs.flatMap((run) => {
    const from = Math.max(span.start, run.start);
    const to = Math.min(span.end, run.end);
    return to - from >= SLIVER_S
      ? [
          {
            played: { end: to, start: from },
            span: recordedSpan(map, from, to),
          },
        ]
      : [];
  });

/** `spans` in played seconds that fall inside `within`, a part's played
 * span, as recorded seconds. */
const recordedWithin = (map: TimeMap, spans: Span[], within: Span) =>
  spans.flatMap(({ end, start }) => {
    const from = Math.max(start, within.start);
    const to = Math.min(end, within.end);
    return to > from ? [recordedSpan(map, from, to)] : [];
  });

/** Lane items in played seconds, as a move dragged lays them out: their
 * parts in recorded seconds, split at the move's `edges` as letting go
 * splits them. An annotation's glides and pin come along within each
 * part. */
function movedLaneItems<
  T extends Span & {
    id: string;
    glides?: TimelineAnnotation["glides"];
    pin?: TimelineAnnotation["pin"];
  },
>(
  items: T[],
  { edges, map, runs }: { edges: number[]; map: TimeMap; runs: Span[] },
): T[] {
  const parts = items.flatMap((item) =>
    recordedParts(map, runs, item).map(({ played, span }, index) => ({
      ...item,
      ...span,
      ...(item.glides && {
        glides: recordedWithin(map, item.glides, played),
      }),
      ...(item.pin && {
        pin: {
          corrections: item.pin.corrections
            .filter((at) => at >= played.start && at <= played.end)
            .map((at) => map.toRecorded(at)),
          outOfView: recordedWithin(map, item.pin.outOfView, played),
          unsure: recordedWithin(map, item.pin.unsure, played),
        },
      }),
      id: partId(item.id, index),
    })),
  );
  return splitSpans(parts, edges);
}

type LaneProps = Pick<
  TimelineStripProps,
  "annotations" | "audio" | "beats" | "map" | "runs" | "scenes"
>;

/** The lanes as the strip draws them, and the scale they are drawn on: in
 * played seconds at rest, and while a move is dragged (`edges` are where
 * it splits what is drawn) as their footage will play on `shown`. The gaps
 * are where the scene rail is out of bounds. */
export function laneLayout(
  props: LaneProps,
  { edges, shown }: { edges: number[]; shown: TimelineScale },
) {
  const { map } = props;
  const gaps = props.beats.flatMap(({ beat, span }) =>
    beat.kind === "gap" ? [span] : [],
  );
  if (edges.length === 0)
    return {
      annotations: props.annotations,
      audio: props.audio,
      gaps: gaps.map((gap) => playedSpan(map, gap)),
      scale: timelineScale(recordedTime(map.duration)),
      scenes: props.scenes,
    };
  const moved = <T extends Span & { id: string }>(items: T[]) =>
    movedLaneItems(items, { edges, map, runs: props.runs });
  return {
    annotations: moved(props.annotations),
    audio: props.audio && {
      ...props.audio,
      carried: props.audio.carried && {
        ...props.audio.carried,
        ghosts: moved(props.audio.carried.ghosts),
      },
      clips: moved(props.audio.clips),
    },
    gaps,
    scale: shown,
    scenes: moved(props.scenes),
  };
}

/** What a lane's dragged end may settle on, in played seconds: the
 * annotations and scenes as they are, and the beats, lulls and cuts where
 * they play (`stripSnap`). */
export const laneSnapParts = (
  props: LaneProps & Pick<TimelineStripProps, "cuts" | "lulls">,
) => ({
  annotations: props.annotations,
  beats: props.beats.map(({ span }) => ({ span: playedSpan(props.map, span) })),
  cuts: props.cuts.map((cut) => playedSpan(props.map, cut)),
  lulls: props.lulls.map(({ span }) => ({ span: playedSpan(props.map, span) })),
  scenes: props.scenes,
});
