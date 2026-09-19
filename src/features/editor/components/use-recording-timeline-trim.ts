// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef } from "react";

import {
  clampRecordingTimelineTrimPosition,
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  RecordingTimelineTrimEdge,
  trimRecordingTimelineSegment,
} from "../recording-timeline-edit";

import {
  nearestTimelineSnapTarget,
  TimelineSnapGesture,
} from "./timeline-snap";

type TrimGesture = {
  boundary: number;
  edge: RecordingTimelineTrimEdge;
  edit: RecordingTimelineEdit;
  outputPosition: number;
  retainedDuration: number;
  segmentId: number;
  snap: TimelineSnapGesture | null;
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
      const reached =
        active.boundary +
        (outputPosition - active.outputPosition) * active.retainedDuration;
      const target = active.snap
        ? nearestTimelineSnapTarget(active.snap, reached)
        : null;
      // A target still lands on a frame: the trim is frame-quantised whatever
      // pulled it there.
      const snapped = snap(target ?? reached);
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
      active.snap?.showGuide(
        target === null
          ? null
          : recordingTimelineSourceToOutput(next, sourcePosition),
      );
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
    ({
      edge,
      outputPosition,
      segmentId,
      snap = null,
    }: {
      edge: RecordingTimelineTrimEdge;
      outputPosition: number;
      segmentId: number;
      snap?: TimelineSnapGesture | null;
    }) => {
      if (!onChange || totalDurationMs <= 0 || activeRef.current) return;
      const segment = edit.segments.find(
        (candidate) => candidate.id === segmentId,
      );
      if (!segment) return;
      selectSegment(segmentId);
      const boundary =
        edge === "start" ? segment.sourceStart : segment.sourceEnd;
      const retainedDuration = recordingTimelineRetainedDuration(edit);
      activeRef.current = {
        boundary,
        edge,
        edit,
        outputPosition,
        retainedDuration,
        segmentId,
        // The dragged edge is not its own target: it would hold the trim
        // where it started until the pointer had travelled the threshold.
        snap: snap && {
          ...snap,
          targets: snap.targets.filter((target) => target !== boundary),
          // The threshold arrives in output time; the trim moves in source.
          threshold: snap.threshold * retainedDuration,
        },
      };
      beginGesture();
      onPreview?.(boundary, "start");
    },
    [beginGesture, edit, onChange, onPreview, selectSegment, totalDurationMs],
  );
  const end = useCallback(
    (outputPosition: number) => {
      if (!activeRef.current) return;
      updateAt(outputPosition, "end");
      requestAnimationFrame(() => {
        activeRef.current?.snap?.showGuide(null);
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
