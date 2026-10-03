// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { RECORDING_VIDEO_TRACK_ORDER } from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { TimelineViewportState } from "../../timeline/timeline-viewport";
import {
  layoutTimedLaneItems,
  StackedLaneFragment,
} from "../../timeline/tracks/timed-lane-layout";
import {
  LaneBox,
  sweptLaneItems,
} from "../../timeline/tracks/timeline-lane-band";

import { RecordingAnnotationClip } from "./recording-annotations";

type LaneClip = RecordingAnnotationClip & { id: string };

/** The narrowest a clip is drawn, in CSS pixels, so a short one can still be
 * taken hold of. */
export const ANNOTATION_CLIP_MINIMUM_WIDTH_PX = 6;

/**
 * The annotation lane's rows, which show the drawing order: a clip sits one
 * row above the highest clip it overlaps and is drawn over, so of any two
 * that overlap, the one in front is the one higher up. A picture's
 * annotations are drawn with it, so every clip on the picture in front sits
 * over every clip on the one behind, as `RECORDING_VIDEO_TRACK_ORDER` stacks
 * them. Clips that never overlap share rows, keeping the lane a single row
 * tall in the common case.
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
  // The runs come in drawing order, bottom first: the picture behind's, then
  // the picture in front's, each in its own order. Each rests on the highest
  // run beneath it that it overlaps; its level counts up from the bottom row.
  const depth = (run: (typeof fragments)[number]) =>
    -RECORDING_VIDEO_TRACK_ORDER.indexOf(run.item.trackId);
  const stacked = [...runs.values()].sort((a, b) => depth(a) - depth(b));
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
): string[] =>
  sweptLaneItems(
    fragments.filter(({ item }) => !item.pin),
    box,
    { laneWidthPx, minimumWidthPx: ANNOTATION_CLIP_MINIMUM_WIDTH_PX, viewport },
  );
