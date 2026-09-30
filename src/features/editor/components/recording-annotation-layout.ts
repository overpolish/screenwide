// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { RecordingAnnotationClip } from "../recording-annotations";
import { RecordingTimelineEdit } from "../recording-timeline-edit";

import {
  layoutTimedLaneItems,
  StackedLaneFragment,
  timedLaneFragmentBox,
} from "./timed-lane-layout";
import {
  timelineFractionToX,
  TimelineViewportState,
} from "./timeline-viewport";

type LaneClip = RecordingAnnotationClip & { id: string };

/** The narrowest a clip is drawn, in CSS pixels, so a short one can still be
 * taken hold of. */
export const ANNOTATION_CLIP_MINIMUM_WIDTH_PX = 6;

/** A rectangle in a lane's own CSS pixels. */
export type LaneBox = {
  bottom: number;
  left: number;
  right: number;
  top: number;
};

/**
 * The annotation lane's rows, which show the drawing order: a clip sits one
 * row above the highest clip it overlaps and is drawn over, so of any two
 * that overlap, the one in front is the one higher up. Clips that never
 * overlap share rows, keeping the lane a single row tall in the common case.
 */
export const recordingAnnotationRows = (
  clips: RecordingAnnotationClip[],
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
): { fragments: StackedLaneFragment<LaneClip>[]; rowCount: number } => {
  const fragments = layoutTimedLaneItems({
    edit,
    items: clips.map((clip) => ({ ...clip, id: clip.annotation.id })),
    sourceDurationMs,
  });
  // Cuts remove time from a clip, not its identity. Stack the whole retained
  // run so an overlapping arrow cannot jump rows at a cut or gain inner grips.
  const runs = new Map<string, (typeof fragments)[number]>();
  for (const fragment of fragments) {
    const previous = runs.get(fragment.item.id);
    if (previous) previous.outputEnd = fragment.outputEnd;
    else
      runs.set(fragment.item.id, { ...fragment, fragmentId: fragment.item.id });
  }
  // The runs come in drawing order, bottom first. Each rests on the highest
  // run beneath it that it overlaps; its level counts up from the bottom row.
  const stacked = [...runs.values()];
  const levels: number[] = [];
  for (const [index, run] of stacked.entries())
    levels.push(
      stacked
        .slice(0, index)
        .reduce(
          (level, under, at) =>
            under.outputStart < run.outputEnd &&
            run.outputStart < under.outputEnd
              ? Math.max(level, levels[at] + 1)
              : level,
          0,
        ),
    );
  const rowCount = Math.max(1, ...levels.map((level) => level + 1));
  return {
    fragments: stacked.map((run, index) => ({
      ...run,
      continuedByNext: false,
      continuesPrevious: false,
      row: rowCount - 1 - levels[index],
      showLabel: true,
    })),
    rowCount,
  };
};

/**
 * The clips a band over `box` touches, as the lane draws `fragments` when it
 * is `laneWidthPx` wide under `viewport`. A pinned clip is never swept up: it
 * follows its content on its own, so it is never chosen with others.
 */
export const sweptAnnotationClips = (
  fragments: StackedLaneFragment<LaneClip>[],
  box: LaneBox,
  {
    laneWidthPx,
    viewport,
  }: { laneWidthPx: number; viewport: TimelineViewportState },
): string[] => {
  const lane = { left: 0, width: laneWidthPx };
  return fragments.flatMap(({ item, outputEnd, outputStart, row }) => {
    if (item.pin) return [];
    const left = timelineFractionToX(outputStart, viewport, lane);
    const right = Math.max(
      timelineFractionToX(outputEnd, viewport, lane),
      left + ANNOTATION_CLIP_MINIMUM_WIDTH_PX,
    );
    const { height, top } = timedLaneFragmentBox(row);
    return left <= box.right &&
      box.left <= right &&
      top <= box.bottom &&
      box.top <= top + height
      ? [item.id]
      : [];
  });
};
