// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { RecordingAnnotationClip } from "../recording-annotations";
import { RecordingTimelineEdit } from "../recording-timeline-edit";

import {
  layoutTimedLaneItems,
  stackTimedLaneFragments,
} from "./timed-lane-layout";
export const recordingAnnotationRows = (
  clips: RecordingAnnotationClip[],
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
) => {
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
  return stackTimedLaneFragments([...runs.values()]);
};
