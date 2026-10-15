// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import {
  recordingTimelineLengthMs,
  recordingTimelineMsAt,
} from "../../timeline/editing/recording-timeline-time";
import { Playhead } from "../../timeline/scrub-playhead";
import {
  insertTimedSpanClip,
  timedSpanClipAt,
} from "../../timeline/tracks/timed-span-clips";
import { useTimedSpanSelection } from "../../timeline/tracks/use-timed-span-selection";

import {
  CameraBackgroundControls,
  usePublishCameraBackground,
} from "./camera-background-channel";
import {
  CAMERA_BACKGROUND_RANGE_MINIMUM_MS,
  CameraBackgroundRange,
} from "./camera-background-ranges";
import { useCameraMattePending } from "./use-camera-matte-pending";

import type { CameraBackgroundLane } from "../../timeline/tracks/camera-background-strip";

const NO_RANGES: CameraBackgroundRange[] = [];
/** How long a range added at the playhead lasts, where there is room. */
const ADDED_RANGE_MS = 10_000;

/**
 * The camera's background ranges and the edits the lane and the camera
 * panel make to them. The panel shows the range chosen in the lane, or the
 * one under the playhead, and adds one there where there is none: over the
 * whole video where the camera has no range yet. Every change is a timeline
 * edit, so it is saved with the recording and undone with the rest of it.
 * The lane fades the parts of the ranges whose matte is still being made.
 */
export function useCameraBackgroundRanges({
  artifactId,
  edit,
  getPositionMs,
  onEdit,
  playhead,
  sourceDurationMs,
}: {
  artifactId: number;
  edit: RecordingTimelineEdit;
  /** Where the playhead is now, in source time. */
  getPositionMs: () => number;
  onEdit: (edit: RecordingTimelineEdit) => void;
  playhead: Pick<Playhead, "subscribe">;
  sourceDurationMs: number;
}): CameraBackgroundLane {
  const ranges = edit.cameraBackgroundRanges ?? NO_RANGES;
  const pending = useCameraMattePending(artifactId, edit);
  // An edit asked for while the panel holds an older render lands on the
  // latest one.
  const editRef = useRef(edit);
  editRef.current = edit;
  const timelineMsNow = () =>
    recordingTimelineMsAt(editRef.current, sourceDurationMs, getPositionMs());
  // The range under the playhead, read again as it moves: the panel and the
  // choice follow it.
  const [currentId, setCurrentId] = useState<string | null>(null);
  const readCurrentRef = useRef(() => {});
  readCurrentRef.current = () => {
    setCurrentId(timedSpanClipAt(ranges, timelineMsNow())?.id ?? null);
  };
  useEffect(() => {
    readCurrentRef.current();
    return playhead.subscribe(() => {
      readCurrentRef.current();
    });
  }, [playhead, ranges]);
  // A press that changes nothing, a drag let go where it began, must leave
  // nothing to undo.
  const commit = (next: CameraBackgroundRange[]) => {
    const latest = editRef.current;
    const before = latest.cameraBackgroundRanges ?? NO_RANGES;
    if (JSON.stringify(next) === JSON.stringify(before)) return;
    onEdit({
      ...latest,
      cameraBackgroundRanges: next.length > 0 ? next : undefined,
    });
  };
  const selection = useTimedSpanSelection({
    clips: ranges,
    commit,
    currentId,
  });
  const chosenId = selection.ids.size === 1 ? [...selection.ids][0] : null;
  const shown =
    ranges.find((range) => range.id === (chosenId ?? currentId)) ?? null;
  const controls: CameraBackgroundControls = {
    setBackground: (background) => {
      if (background === "kept") {
        if (shown) commit(ranges.filter((range) => range.id !== shown.id));
        return;
      }
      if (shown) {
        commit(
          ranges.map((range) =>
            range.id === shown.id ? { ...range, background } : range,
          ),
        );
        return;
      }
      const lengthMs = Math.round(
        recordingTimelineLengthMs(editRef.current, sourceDurationMs),
      );
      const id = crypto.randomUUID();
      if (ranges.length === 0) {
        if (lengthMs > 0)
          commit([{ background, endMs: lengthMs, id, startMs: 0 }]);
        return;
      }
      const added = insertTimedSpanClip({
        atMs: timelineMsNow(),
        clips: ranges,
        defaultMs: ADDED_RANGE_MS,
        lengthMs,
        make: (span): CameraBackgroundRange => ({ ...span, background, id }),
        minimumMs: CAMERA_BACKGROUND_RANGE_MINIMUM_MS,
      });
      if (added) commit(added.clips);
    },
  };
  usePublishCameraBackground(
    { hasRanges: ranges.length > 0, range: shown },
    controls,
  );
  return {
    onChange: commit,
    onClearSelection: selection.onClear,
    onSelect: selection.onSelect,
    onSelectSwept: selection.onSelectSwept,
    pendingSpans: pending.length > 0 ? () => pending : undefined,
    ranges,
    selectedIds: selection.ids,
  };
}
