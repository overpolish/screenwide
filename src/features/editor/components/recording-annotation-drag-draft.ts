// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What the annotation lane and the preview show while a clip is dragged.

import { arranged } from "../annotation-order";
import { moveRecordingAnnotationClip } from "../recording-annotation-geometry";
import {
  RecordingAnnotationClip,
  recordingAnnotationClipsMeet,
  renumberedAnnotationClips,
} from "../recording-annotations";
import { RecordingTimelineEdit } from "../recording-timeline-edit";

import { TIMED_LANE_ROW_HEIGHT_PX } from "./timed-lane-layout";

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
 * `clips` with the one named `id` carried `rows` rows up the lane, or down
 * for a negative count: each row brings it over the next clip it overlaps, or
 * sends it under the previous one, until there is none left to pass.
 */
const carriedThroughRows = (
  clips: RecordingAnnotationClip[],
  id: string,
  rows: number,
) => {
  let carried = clips;
  for (let step = 0; step < Math.abs(rows); step += 1) {
    const next = arranged(
      carried,
      carried.findIndex((clip) => clip.annotation.id === id),
      {
        arrangement: rows > 0 ? "forward" : "backward",
        meets: recordingAnnotationClipsMeet,
      },
    );
    if (next === carried) break;
    carried = next;
  }
  return carried;
};

/**
 * Which way a carried clip may go. Held Shift locks the drag to the way the
 * pointer has come further - sideways in time, or up and down through the
 * drawing order - so a clip can be restacked without losing its timing, or
 * slid in time without slipping a row.
 */
export const carriedAxes = (
  deltaX: number,
  deltaY: number,
  locked: boolean,
) => ({
  order: !locked || Math.abs(deltaY) > Math.abs(deltaX),
  time: !locked || Math.abs(deltaX) >= Math.abs(deltaY),
});

/**
 * The clips while the one named `id` is carried by its body: moved
 * `deltaOutput` along the output timeline and `lift` pixels up the lane, a
 * row counting once the pointer has come three quarters of the way across it
 * so a sideways drag that wanders keeps its place. Counters are numbered as
 * they will be on release, so the lane and the preview show it as it goes.
 */
export const carriedClips = ({
  deltaOutput,
  edit,
  id,
  lift,
  original,
  sourceDurationMs,
}: {
  deltaOutput: number;
  edit: RecordingTimelineEdit;
  id: string;
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
  const moved = original.map((clip) =>
    clip.annotation.id === id
      ? moveRecordingAnnotationClip({
          clip,
          deltaOutput,
          edit,
          sourceDurationMs,
        })
      : clip,
  );
  return renumberedAnnotationClips(
    carriedThroughRows(moved, id, rows),
    original,
  );
};
