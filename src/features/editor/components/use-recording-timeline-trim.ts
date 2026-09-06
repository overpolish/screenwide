// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef } from "react";

import {
  clampRecordingTimelineTrimPosition,
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
  RecordingTimelineTrimEdge,
  trimRecordingTimelineSegment,
} from "../recording-timeline-edit";

type TrimGesture = {
  boundary: number;
  edge: RecordingTimelineTrimEdge;
  edit: RecordingTimelineEdit;
  outputPosition: number;
  retainedDuration: number;
  segmentId: number;
};

export function useRecordingTimelineTrim({
  beginGesture,
  edit,
  endGesture,
  framesPerSecond,
  onChange,
  onPreview,
  onRestorePreview,
  selectSegment,
  snap,
  totalDurationMs,
}: {
  beginGesture: () => void;
  edit: RecordingTimelineEdit;
  endGesture: () => void;
  framesPerSecond: number | null;
  selectSegment: (segmentId: number) => void;
  snap: (sourcePosition: number) => number;
  totalDurationMs: number;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onPreview?: (sourcePosition: number, phase: "end" | "move" | "start") => void;
  onRestorePreview?: () => void;
}) {
  const activeRef = useRef<TrimGesture | null>(null);
  const updateAt = useCallback(
    (outputPosition: number, phase: "end" | "move") => {
      const active = activeRef.current;
      if (!active || !onChange || totalDurationMs <= 0) return null;
      const frameDurationMs =
        framesPerSecond !== null &&
        Number.isFinite(framesPerSecond) &&
        framesPerSecond > 0
          ? 1_000 / framesPerSecond
          : 1;
      const snapped = snap(
        active.boundary +
          (outputPosition - active.outputPosition) * active.retainedDuration,
      );
      const sourcePosition = clampRecordingTimelineTrimPosition(active.edit, {
        edge: active.edge,
        minimumDuration: frameDurationMs / totalDurationMs,
        segmentId: active.segmentId,
        sourcePosition: snapped,
      });
      const next = trimRecordingTimelineSegment(active.edit, {
        edge: active.edge,
        minimumDuration: frameDurationMs / totalDurationMs,
        segmentId: active.segmentId,
        sourcePosition,
      });
      if (next !== edit) onChange(next);
      onPreview?.(sourcePosition, phase);
      return sourcePosition === snapped
        ? null
        : active.outputPosition +
            (sourcePosition - active.boundary) / active.retainedDuration;
    },
    [edit, framesPerSecond, onChange, onPreview, snap, totalDurationMs],
  );
  const update = useCallback(
    (outputPosition: number) => updateAt(outputPosition, "move"),
    [updateAt],
  );
  const begin = useCallback(
    (
      segmentId: number,
      edge: RecordingTimelineTrimEdge,
      outputPosition: number,
    ) => {
      if (!onChange || totalDurationMs <= 0 || activeRef.current) return;
      const segment = edit.segments.find(
        (candidate) => candidate.id === segmentId,
      );
      if (!segment) return;
      selectSegment(segmentId);
      activeRef.current = {
        boundary: edge === "start" ? segment.sourceStart : segment.sourceEnd,
        edge,
        edit,
        outputPosition,
        retainedDuration: recordingTimelineRetainedDuration(edit),
        segmentId,
      };
      beginGesture();
      onPreview?.(
        edge === "start" ? segment.sourceStart : segment.sourceEnd,
        "start",
      );
    },
    [beginGesture, edit, onChange, onPreview, selectSegment, totalDurationMs],
  );
  const end = useCallback(
    (outputPosition: number) => {
      if (!activeRef.current) return;
      updateAt(outputPosition, "end");
      requestAnimationFrame(() => {
        activeRef.current = null;
        onRestorePreview?.();
        endGesture();
      });
    },
    [endGesture, onRestorePreview, updateAt],
  );

  return {
    begin,
    end,
    isActive: () => activeRef.current !== null,
    update,
  };
}
