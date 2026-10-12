// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use } from "react";

import { RecordingAnnotationClip } from "../recording/annotations/recording-annotations";
import { RecordingKeyboardTimelineItem } from "../types";

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineSourceToOutput,
} from "./editing/recording-timeline-edit";
import { recordingTimelineLengthMs } from "./editing/recording-timeline-time";
import { layoutTimedLaneItems } from "./tracks/timed-lane-layout";

/** How close, on screen, a dragged edge has to come to a target to take it. */
export const TIMELINE_SNAP_THRESHOLD_PX = 8;

/** A place a drag may land on, both where it plays, from 0 to 1 along the
 * output timeline, and where it is in the recording, from 0 to 1 along it. */
export type TimelineSnapTarget = { output: number; source: number };

/**
 * The snap mode as the lanes see it. A drag asks whether to snap when it
 * moves rather than when it begins: the toggle is what the toolbar shows, and
 * holding Alt inverts it for as long as it is held, so an edge can be pulled
 * off a target part way through the gesture.
 */
export type TimelineSnap = {
  /** The places a drag may land on, gathered as the drag begins. The
   * annotations being dragged are not targets for themselves. */
  gather: (excludeAnnotationIds?: ReadonlySet<string>) => TimelineSnapTarget[];
  isActive: boolean;
  isSnapping: () => boolean;
  /** Where the held drag is snapped to, in output time; null once it is not. */
  showGuide: (outputPosition: number | null) => void;
};

export const TimelineSnapContext = createContext<TimelineSnap>({
  gather: () => [],
  isActive: false,
  isSnapping: () => false,
  showGuide: () => undefined,
});

export const useTimelineSnap = () => use(TimelineSnapContext);

/**
 * What one drag holds on to. The targets and the threshold are in whichever
 * time the drag itself works in, `space`: output for a clip on the
 * timeline, source for a trim of the footage.
 */
export type TimelineSnapGesture = Pick<
  TimelineSnap,
  "isSnapping" | "showGuide"
> & {
  targets: number[];
  threshold: number;
};

export const beginTimelineSnapGesture = (
  snap: TimelineSnap,
  {
    excludeAnnotationIds,
    space,
    threshold,
  }: {
    space: keyof TimelineSnapTarget;
    threshold: number;
    excludeAnnotationIds?: ReadonlySet<string>;
  },
): TimelineSnapGesture => ({
  isSnapping: snap.isSnapping,
  showGuide: snap.showGuide,
  targets: snap.gather(excludeAnnotationIds).map((target) => target[space]),
  threshold,
});

/** The target within reach of `position`, the closest one when several are. */
export function nearestTimelineSnapTarget(
  { isSnapping, targets, threshold }: TimelineSnapGesture,
  position: number,
): number | null {
  if (!isSnapping()) return null;
  let nearest: number | null = null;
  let distance = threshold;
  for (const target of targets) {
    const candidate = Math.abs(target - position);
    if (candidate <= distance) {
      nearest = target;
      distance = candidate;
    }
  }
  return nearest;
}

/**
 * The shift that lands either end of a moved range on a target: whichever
 * end is nearer to one wins, so a clip can be butted up against its
 * neighbour from either side.
 */
export function timelineSnapRangeShift(
  gesture: TimelineSnapGesture,
  start: number,
  end: number,
): { shift: number; target: number } | null {
  const startTarget = nearestTimelineSnapTarget(gesture, start);
  const endTarget = nearestTimelineSnapTarget(gesture, end);
  const startShift = startTarget === null ? null : startTarget - start;
  const endShift = endTarget === null ? null : endTarget - end;
  if (startShift === null && endShift === null) return null;
  if (
    endShift === null ||
    (startShift !== null && Math.abs(startShift) <= Math.abs(endShift))
  )
    return { shift: startShift ?? 0, target: startTarget ?? start };
  return { shift: endShift, target: endTarget ?? end };
}

/**
 * Every place a drag may snap to: the edges of the other annotations, of
 * the shortcut badges the lane shows, of the retained segments, the moments
 * the edit keeps, and the playhead. Moments in cut-away source still count:
 * one there maps onto the cut it sits in, and a trim may reach it.
 */
export function timelineSnapTargets({
  annotationClips,
  edit,
  excludeAnnotationIds,
  hiddenKeyboardFragmentIds,
  hiddenKeyboardItemIds,
  keyboardItems,
  momentSources = [],
  playheadOutput,
  sourceDurationMs,
}: {
  annotationClips: RecordingAnnotationClip[];
  edit: RecordingTimelineEdit;
  hiddenKeyboardFragmentIds: ReadonlySet<string>;
  hiddenKeyboardItemIds: ReadonlySet<number>;
  keyboardItems: RecordingKeyboardTimelineItem[];
  playheadOutput: number;
  sourceDurationMs: number;
  excludeAnnotationIds?: ReadonlySet<string>;
  /** Where the visible moments are, 0 to 1 along the recording. */
  momentSources?: readonly number[];
}): TimelineSnapTarget[] {
  const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
  if (lengthMs <= 0) return [];
  const atOutput = (output: number) => ({
    output,
    source: recordingTimelineOutputToSource(edit, output),
  });
  const targets: TimelineSnapTarget[] = [];
  for (const segment of layoutRecordingTimelineSegments(edit))
    targets.push(
      { output: segment.outputStart, source: segment.sourceStart },
      { output: segment.outputEnd, source: segment.sourceEnd },
    );
  for (const clip of annotationClips)
    if (!excludeAnnotationIds?.has(clip.annotation.id))
      targets.push(
        atOutput(clip.startMs / lengthMs),
        atOutput(clip.endMs / lengthMs),
      );
  // Laid out the way the lane draws them, so a badge hidden from the lane is
  // not a target either.
  for (const fragment of layoutTimedLaneItems({
    edit,
    items: keyboardItems,
    sourceDurationMs,
  }))
    if (
      !hiddenKeyboardFragmentIds.has(fragment.fragmentId) &&
      !hiddenKeyboardItemIds.has(fragment.item.id)
    )
      targets.push(
        atOutput(fragment.outputStart),
        atOutput(fragment.outputEnd),
      );
  for (const source of momentSources)
    targets.push({
      output: recordingTimelineSourceToOutput(edit, source),
      source,
    });
  targets.push(atOutput(playheadOutput));
  return targets;
}
