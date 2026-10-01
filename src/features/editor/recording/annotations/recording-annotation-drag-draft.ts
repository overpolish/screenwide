// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What the annotation lane and the preview show while a clip is dragged.

import { arrangedGroup } from "../../annotations/annotation-order";
import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
} from "../../timeline/editing/recording-timeline-edit";
import {
  TimelineSnapGesture,
  timelineSnapRangeShift,
} from "../../timeline/timeline-snap";
import { TIMED_LANE_ROW_HEIGHT_PX } from "../../timeline/tracks/timed-lane-layout";

import { moveRecordingAnnotationClip } from "./recording-annotation-geometry";
import {
  RecordingAnnotationClip,
  recordingAnnotationClipsMeet,
  renumberedAnnotationClips,
} from "./recording-annotations";

/**
 * The clips as the preview should show them while `id` is being dragged: that
 * annotation drawn whole rather than at the reveal its clip's bounds put it at.
 *
 * A trim handle sits exactly where the annotation is arriving or leaving, so
 * the frame it seeks to is the one frame where the annotation is barely there -
 * which is no use at all for deciding where the handle belongs. Marking the
 * clip as not animated is how an annotation is drawn whole everywhere else, and
 * it is the preview's own copy: the list that reaches the document keeps its
 * animation.
 */
export const previewedWhole = (
  clips: RecordingAnnotationClip[],
  id: string,
): RecordingAnnotationClip[] =>
  clips.map((clip) =>
    clip.annotation.id === id && clip.annotation.animated
      ? { ...clip, annotation: { ...clip.annotation, animated: false } }
      : clip,
  );

/**
 * `clips` with the ones named in `ids` carried `rows` rows up the lane, or
 * down for a negative count, until none of them has anything left to pass.
 * Each row is one step of the group through the drawing order, so the group
 * keeps its own stacking.
 */
const carriedThroughRows = (
  clips: RecordingAnnotationClip[],
  ids: ReadonlySet<string>,
  rows: number,
) => {
  let carried = clips;
  for (let step = 0; step < Math.abs(rows); step += 1) {
    const next = arrangedGroup(carried, (clip) => ids.has(clip.annotation.id), {
      arrangement: rows > 0 ? "forward" : "backward",
      meets: recordingAnnotationClipsMeet,
    });
    if (next === carried) break;
    carried = next;
  }
  return carried;
};

/** Where the clips named in `ids` begin and end together on the output
 * timeline, or null where none of them is in `clips`. */
const carriedSpan = (
  clips: RecordingAnnotationClip[],
  ids: ReadonlySet<string>,
  {
    edit,
    sourceDurationMs,
  }: { edit: RecordingTimelineEdit; sourceDurationMs: number },
) => {
  if (sourceDurationMs <= 0) return null;
  let span: { end: number; start: number } | null = null;
  for (const clip of clips) {
    if (!ids.has(clip.annotation.id)) continue;
    const start = recordingTimelineSourceToOutput(
      edit,
      clip.startMs / sourceDurationMs,
    );
    const end = recordingTimelineSourceToOutput(
      edit,
      clip.endMs / sourceDurationMs,
    );
    span = span
      ? { end: Math.max(span.end, end), start: Math.min(span.start, start) }
      : { end, start };
  }
  return span;
};

/**
 * Which way a carried clip may go. Held Shift locks the drag to the way the
 * pointer has come further - sideways in time, or up and down through the
 * drawing order - so a clip can be restacked without losing its timing, or
 * slid in time without slipping a row.
 */
const carriedAxes = (deltaX: number, deltaY: number, locked: boolean) => ({
  order: !locked || Math.abs(deltaY) > Math.abs(deltaX),
  time: !locked || Math.abs(deltaX) >= Math.abs(deltaY),
});

/**
 * The clips while the ones named in `ids` are carried by a body: moved
 * `deltaOutput` along the output timeline together, stopping as one where the
 * first reaches the start or the last reaches the end, and `lift` pixels up
 * the lane, a row counting once the pointer has come three quarters of the
 * way across it so a sideways drag that wanders keeps its place. Counters are
 * numbered as they will be on release, so the lane and the preview show it as
 * it goes.
 */
export const carriedClips = ({
  deltaOutput,
  edit,
  ids,
  lift,
  original,
  sourceDurationMs,
}: {
  deltaOutput: number;
  edit: RecordingTimelineEdit;
  ids: ReadonlySet<string>;
  lift: number;
  original: RecordingAnnotationClip[];
  sourceDurationMs: number;
}) => {
  const rows =
    Math.sign(lift) *
    Math.trunc(
      (Math.abs(lift) + TIMED_LANE_ROW_HEIGHT_PX / 4) /
        TIMED_LANE_ROW_HEIGHT_PX,
    );
  const span = carriedSpan(original, ids, { edit, sourceDurationMs });
  const shift = span
    ? Math.max(-span.start, Math.min(1 - span.end, deltaOutput))
    : 0;
  const moved = original.map((clip) =>
    ids.has(clip.annotation.id)
      ? moveRecordingAnnotationClip({
          clip,
          deltaOutput: shift,
          edit,
          sourceDurationMs,
        })
      : clip,
  );
  return renumberedAnnotationClips(
    carriedThroughRows(moved, ids, rows),
    original,
  );
};

/**
 * What a body drag shows once the pointer has come `deltaX`, `deltaY` pixels
 * from its press: the carried clips, and the snap target the group's start or
 * end landed on. `outputWidthPx` is how many pixels the whole output timeline
 * spans at the current zoom. Null where none of `ids` is among the clips.
 */
export const carriedDraft = ({
  deltaX,
  deltaY,
  edit,
  ids,
  locked,
  original,
  outputWidthPx,
  snap,
  sourceDurationMs,
}: {
  deltaX: number;
  deltaY: number;
  edit: RecordingTimelineEdit;
  ids: ReadonlySet<string>;
  locked: boolean;
  original: RecordingAnnotationClip[];
  outputWidthPx: number;
  snap: TimelineSnapGesture;
  sourceDurationMs: number;
}) => {
  const span = carriedSpan(original, ids, { edit, sourceDurationMs });
  if (!span) return null;
  const axes = carriedAxes(deltaX, deltaY, locked);
  const shift = axes.time ? deltaX / outputWidthPx : 0;
  const snapped = axes.time
    ? timelineSnapRangeShift(snap, span.start + shift, span.end + shift)
    : null;
  return {
    clips: carriedClips({
      deltaOutput: shift + (snapped?.shift ?? 0),
      edit,
      ids,
      lift: axes.order ? -deltaY : 0,
      original,
      sourceDurationMs,
    }),
    target: snapped?.target ?? null,
  };
};
