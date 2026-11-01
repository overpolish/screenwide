// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  recordingTimelineCuts,
  restoreRecordingTimelineCut,
} from "../editing/recording-timeline-cuts";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { moveRecordingTimelineFootage } from "../editing/recording-timeline-footage-move";
import { type Span } from "../timeline-scale";

import { cutAt, joinedAt, removed, restored } from "./recording-strip-source";
import { cutId, jumpIndexes, sourceRanks } from "./recording-strip-time";

import type { RecordingTimelineSegment } from "../recording-timeline-types";

// What each change made on the strip does to the recording's timeline
// edit, as the strip's stories change their own content: a cut takes a
// stretch out wherever it plays, a trim moves where a beat or lull starts
// or ends playing, a seam puts back what it stands for, the scissors split,
// a move plays stretches elsewhere and a jump pressed puts one back. Times
// are recorded seconds, `recorded` long; the edit counts in shares of it.

/** The least of a beat or lull a trim leaves, in seconds. */
const LEAST_KEPT_S = 0.1;

/** `edit` without the stretch `span`. */
export const cutStretch = (
  edit: RecordingTimelineEdit,
  span: Span,
  recorded: number,
) => removed(edit, span.start / recorded, span.end / recorded);

/** `next`, a step of a drag, with the annotations, scenes and camera
 * background ranges of `start`, the edit the drag began on. Each step is
 * made from the one before it, but its lists are carried onto the step's
 * timeline from where they stood when the drag began: carried step by
 * step, a clip would drift by the rounding of every step to whole ms, and
 * lose for good what a step took out and a later one gave back. */
export const withListsOf = (
  start: RecordingTimelineEdit,
  next: RecordingTimelineEdit,
): RecordingTimelineEdit => ({
  ...next,
  annotationClips: start.annotationClips,
  cameraBackgroundRanges: start.cameraBackgroundRanges,
  sceneClips: start.sceneClips,
});

/** `edit` with the part of `span` that plays, `kept` now, starting or ending
 * at `at`: what is trimmed is cut, and what a trim lets go of plays again. */
export function trimmed(
  edit: RecordingTimelineEdit,
  {
    at,
    edge,
    kept,
    span,
  }: {
    at: number;
    edge: "end" | "start";
    kept: Span;
    span: Span;
  },
  recorded: number,
): RecordingTimelineEdit {
  const share = (seconds: number) => seconds / recorded;
  if (edge === "start") {
    const to = Math.min(Math.max(at, span.start), kept.end - LEAST_KEPT_S);
    return to < kept.start
      ? restored(edit, share(to), share(kept.start))
      : removed(edit, share(kept.start), share(to));
  }
  const to = Math.max(Math.min(at, span.end), kept.start + LEAST_KEPT_S);
  return to > kept.end
    ? restored(edit, share(kept.end), share(to))
    : removed(edit, share(to), share(kept.end));
}

/** `edit` playing again the cuts named `ids`. */
export function restoredCuts(
  edit: RecordingTimelineEdit,
  ids: string[],
): RecordingTimelineEdit {
  return ids.reduce((current, id) => {
    const cut = recordingTimelineCuts(current).find(
      (each) => cutId(each.sourceStart) === id,
    );
    if (!cut) return current;
    const next = restoreRecordingTimelineCut(current, cut);
    return next === current
      ? restored(current, cut.sourceStart, cut.sourceEnd)
      : next;
  }, edit);
}

/** `edit` split at the recorded moment `at`. */
export const splitAt = (
  edit: RecordingTimelineEdit,
  at: number,
  recorded: number,
) => cutAt(edit, at / recorded);

/** `edit` no longer split at the recorded moment `at`. */
export const unsplitAt = (
  edit: RecordingTimelineEdit,
  at: number,
  recorded: number,
) => joinedAt(edit, [at / recorded]);

/** `edit` with `spans`, in the order they play, playing together just
 * before the recorded moment `before`, or at the end for null. */
export function moved(
  edit: RecordingTimelineEdit,
  { before, spans }: { before: number | null; spans: Span[] },
  recorded: number,
): RecordingTimelineEdit {
  const ms = (seconds: number) => seconds * 1000;
  const next = spans.reduce(
    (current, span) =>
      moveRecordingTimelineFootage(
        current,
        {
          beforeMs: before === null ? null : ms(before),
          endMs: ms(span.end),
          startMs: ms(span.start),
        },
        ms(recorded),
      ),
    edit,
  );
  if (next === edit) return edit;
  return joinedAt(next, [
    ...spans.flatMap(({ end, start }) => [start / recorded, end / recorded]),
    ...(before === null ? [] : [before / recorded]),
  ]);
}

/** The run of segments around `index` that play as they were recorded. */
function runAround(
  segments: readonly RecordingTimelineSegment[],
  index: number,
) {
  const jumps = new Set(jumpIndexes(segments));
  let first = index;
  while (first > 0 && !jumps.has(first)) first -= 1;
  let last = index;
  while (last + 1 < segments.length && !jumps.has(last + 1)) last += 1;
  return { first, last };
}

/** `segments` with the run from `first` to `last` played where it was
 * recorded: after the segment recorded just before it. */
function runBack(
  segments: readonly RecordingTimelineSegment[],
  { first, last }: { first: number; last: number },
) {
  const run = segments.slice(first, last + 1);
  const rest = [...segments.slice(0, first), ...segments.slice(last + 1)];
  const ranks = sourceRanks(segments);
  const rank = ranks.get(segments[first].id) ?? 0;
  const after = rest.findIndex((segment) => ranks.get(segment.id) === rank - 1);
  return [...rest.slice(0, after + 1), ...run, ...rest.slice(after + 1)];
}

/** `edit` with the jump to the recorded moment `at` put right: of the two
 * runs either side of it, the one whose going back leaves fewer jumps goes
 * back where it was recorded, the shorter where they tie, so pressing any
 * jump a move made undoes it. */
export function putBack(
  edit: RecordingTimelineEdit,
  at: number,
  recorded: number,
): RecordingTimelineEdit {
  const { segments } = edit;
  const index = segments.findIndex(
    (segment, each) =>
      each > 0 && Math.abs(segment.sourceStart * recorded - at) < 1e-6,
  );
  if (index < 1) return edit;
  const [best] = [index - 1, index]
    .map((each) => {
      const run = runAround(segments, each);
      const moving = segments.slice(run.first, run.last + 1);
      const order = runBack(segments, run);
      return {
        jumps: jumpIndexes(order).length,
        length: moving.reduce(
          (sum, { sourceEnd, sourceStart }) => sum + sourceEnd - sourceStart,
          0,
        ),
        order,
      };
    })
    .sort((one, other) => one.jumps - other.jumps || one.length - other.length);
  // Where play jumped before is where moves cut the segments; those that
  // now play on as recorded are made one again.
  const edges = jumpIndexes(segments).flatMap((each) => [
    segments[each - 1].sourceEnd,
    segments[each].sourceStart,
  ]);
  return joinedAt({ ...edit, segments: best.order }, edges);
}
