// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import {
  RecordingTimelineChange,
  retimeRecordingTimelineClips,
} from "../timeline/editing/recording-timeline-time";
import { TimedSpanClip } from "../timeline/tracks/timed-span-clips";

import {
  RecordingAnnotationClip,
  renumberedAnnotationClips,
} from "./annotations/recording-annotations";

import type { RecordingTimelineSegment } from "../timeline/recording-timeline-types";

const sameSegments = (
  a: readonly RecordingTimelineSegment[],
  b: readonly RecordingTimelineSegment[],
) =>
  a === b ||
  (a.length === b.length &&
    a.every(
      (segment, index) =>
        segment.sourceStart === b[index].sourceStart &&
        segment.sourceEnd === b[index].sourceEnd &&
        (segment.playbackRate ?? 1) === (b[index].playbackRate ?? 1),
    ));

/** `clips` carried across `change`. A part after the first takes an id of
 * its own; counters are numbered again, since their order on the timeline
 * may have changed. */
const retimedAnnotationClips = (
  clips: RecordingAnnotationClip[],
  change: RecordingTimelineChange,
): RecordingAnnotationClip[] => {
  const retimed = retimeRecordingTimelineClips(
    clips,
    change,
    (clip, { endMs, startMs }, index) => ({
      ...clip,
      annotation:
        index === 0
          ? clip.annotation
          : { ...clip.annotation, id: crypto.randomUUID() },
      endMs,
      startMs,
    }),
  );
  return retimed === clips
    ? clips
    : renumberedAnnotationClips([...retimed], clips);
};

/** `clips`, scenes or camera background ranges, carried across `change`,
 * kept in timeline order. A part after the first takes an id of its own. */
export const retimedSpanClips = <Clip extends TimedSpanClip>(
  clips: Clip[],
  change: RecordingTimelineChange,
): Clip[] => {
  const retimed = retimeRecordingTimelineClips(
    clips,
    change,
    (clip, { endMs, startMs }, index) => ({
      ...clip,
      endMs,
      id: index === 0 ? clip.id : crypto.randomUUID(),
      startMs,
    }),
  );
  return retimed === clips
    ? clips
    : [...retimed].sort((a, b) => a.startMs - b.startMs);
};

/** The segments each list of clips is timed on, for the lists edits have
 * carried. Lists are never changed in place, so a list is timed on one
 * timeline for as long as it lives. */
export type RecordingOverlayTimelines = WeakMap<
  object,
  readonly RecordingTimelineSegment[]
>;

/**
 * `next`, an edit of `previous`, with its annotations, scenes and camera
 * background ranges carried onto its timeline. Clips are timed on the edited
 * video, so an edit that
 * changes the segments re-times each clip from the timeline its list was
 * timed on: footage it cut away shortens or removes the clip for good, and
 * footage it moved moves the clip with it. A gesture that commits edit after
 * edit made from the one it started on is re-timed from that one each time,
 * so its clips never drift, and footage it takes away and gives back before
 * letting go leaves them whole. A list the edit made itself is timed on its
 * own timeline.
 */
export function retimeRecordingOverlays(
  {
    next,
    previous,
  }: { next: RecordingTimelineEdit; previous: RecordingTimelineEdit },
  {
    sourceDurationMs,
    timelines,
  }: { sourceDurationMs: number; timelines: RecordingOverlayTimelines },
): RecordingTimelineEdit {
  const timedOn = (
    list: object | undefined,
    previousList: object | undefined,
  ) => {
    if (!list) return next.segments;
    const segments =
      timelines.get(list) ??
      (list === previousList && previous.artifactId === next.artifactId
        ? previous.segments
        : next.segments);
    timelines.set(list, segments);
    return segments;
  };
  const changeFrom = (segments: readonly RecordingTimelineSegment[]) =>
    sameSegments(segments, next.segments)
      ? null
      : { next: next.segments, previous: segments, sourceDurationMs };
  const annotationChange = changeFrom(
    timedOn(next.annotationClips, previous.annotationClips),
  );
  const sceneChange = changeFrom(timedOn(next.sceneClips, previous.sceneClips));
  const rangeChange = changeFrom(
    timedOn(next.cameraBackgroundRanges, previous.cameraBackgroundRanges),
  );
  const annotationClips =
    next.annotationClips && annotationChange
      ? retimedAnnotationClips(next.annotationClips, annotationChange)
      : next.annotationClips;
  const sceneClips =
    next.sceneClips && sceneChange
      ? retimedSpanClips(next.sceneClips, sceneChange)
      : next.sceneClips;
  const cameraBackgroundRanges =
    next.cameraBackgroundRanges && rangeChange
      ? retimedSpanClips(next.cameraBackgroundRanges, rangeChange)
      : next.cameraBackgroundRanges;
  const result =
    annotationClips === next.annotationClips &&
    sceneClips === next.sceneClips &&
    cameraBackgroundRanges === next.cameraBackgroundRanges
      ? next
      : { ...next, annotationClips, cameraBackgroundRanges, sceneClips };
  recordOverlayTimeline(result, timelines);
  return result;
}

/** Notes that `edit`'s lists are timed on its own timeline, as those of an
 * edit brought back from history are. */
export function recordOverlayTimeline(
  edit: RecordingTimelineEdit,
  timelines: RecordingOverlayTimelines,
) {
  if (edit.annotationClips) timelines.set(edit.annotationClips, edit.segments);
  if (edit.sceneClips) timelines.set(edit.sceneClips, edit.segments);
  if (edit.cameraBackgroundRanges)
    timelines.set(edit.cameraBackgroundRanges, edit.segments);
}
