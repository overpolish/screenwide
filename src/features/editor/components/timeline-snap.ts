// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use } from "react";

import { RecordingAnnotationClip } from "../recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
} from "../recording-timeline-edit";
import { RecordingKeyboardTimelineItem } from "../types";

import { layoutTimedLaneItems } from "./timed-lane-layout";

/** How close, on screen, a dragged edge has to come to a target to take it. */
export const TIMELINE_SNAP_THRESHOLD_PX = 8;

/**
 * The snap mode as the lanes see it. A drag asks whether to snap when it
 * moves rather than when it begins: the toggle is what the toolbar shows, and
 * holding Alt inverts it for as long as it is held, so an edge can be pulled
 * off a target part way through the gesture.
 */
export type TimelineSnap = {
  /** Source positions a drag may land on, gathered as the drag begins. */
  gather: (excludeAnnotationId?: string) => number[];
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
 * time the drag itself works in - output for an annotation, source for a
 * trim - so `mapTarget` converts them when the drag begins.
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
    excludeAnnotationId,
    mapTarget = (source) => source,
    threshold,
  }: {
    threshold: number;
    excludeAnnotationId?: string;
    mapTarget?: (source: number) => number;
  },
): TimelineSnapGesture => ({
  isSnapping: snap.isSnapping,
  showGuide: snap.showGuide,
  targets: snap.gather(excludeAnnotationId).map(mapTarget),
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
 * Every source position a drag may snap to: the edges of the other
 * annotations, of the shortcut badges the lane shows, of the retained
 * segments, and the playhead. Positions in cut-away source still count: an
 * annotation there maps onto the cut it sits in, and a trim may reach it.
 */
export function timelineSnapTargets({
  annotationClips,
  edit,
  excludeAnnotationId,
  hiddenKeyboardFragmentIds,
  hiddenKeyboardItemIds,
  keyboardItems,
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
  excludeAnnotationId?: string;
}): number[] {
  if (sourceDurationMs <= 0) return [];
  const targets: number[] = [];
  for (const segment of edit.segments)
    targets.push(segment.sourceStart, segment.sourceEnd);
  for (const clip of annotationClips)
    if (clip.annotation.id !== excludeAnnotationId)
      targets.push(
        clip.startMs / sourceDurationMs,
        clip.endMs / sourceDurationMs,
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
        recordingTimelineOutputToSource(edit, fragment.outputStart),
        recordingTimelineOutputToSource(edit, fragment.outputEnd),
      );
  targets.push(recordingTimelineOutputToSource(edit, playheadOutput));
  return targets;
}
