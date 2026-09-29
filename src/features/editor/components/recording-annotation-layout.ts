// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { RecordingAnnotationClip } from "../recording-annotations";
import { RecordingTimelineEdit } from "../recording-timeline-edit";

import { layoutTimedLaneItems, StackedLaneFragment } from "./timed-lane-layout";

type LaneClip = RecordingAnnotationClip & { id: string };

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
