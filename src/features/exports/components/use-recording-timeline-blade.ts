// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { PREVIEW_FRAME_MS } from "../duration";
import { cutKeyboardTimeline } from "../recording-keyboard-timeline-cut";
import {
  createRecordingTimelineEdit,
  deleteRecordingTimelineRange,
  deleteRecordingTimelineSegment,
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  remapRecordingTimelinePosition,
  setRecordingTimelineSegmentPlaybackRate,
  snapRecordingTimelinePosition,
} from "../recording-timeline-edit";
import { setRecordingTimelineRangePlaybackRate } from "../recording-timeline-speed";
import { useExportEditGesture } from "../use-export-edit-history";
import { useExportWindowShortcuts } from "../use-export-window-shortcuts";

import { clamp, Playhead } from "./scrub-playhead";
import { SeekHandler } from "./scrub-timeline";
import { TimelineRangeSelection } from "./timeline-blade";
import { useRecordingTimelineTrim } from "./use-recording-timeline-trim";

export function useRecordingTimelineBlade({
  artifactId,
  edit,
  framesPerSecond,
  getPositionMs,
  onChange,
  onTrimPreviewRestore,
  onTrimPreviewStart,
  playhead,
  seekPlayer,
  shortcutsEnabled,
  totalDurationMs,
}: {
  artifactId: number;
  framesPerSecond: number | null;
  getPositionMs: () => number;
  playhead: Playhead;
  seekPlayer: (positionMs: number, phase: "end" | "move" | "start") => void;
  shortcutsEnabled: boolean;
  totalDurationMs: number;
  edit?: RecordingTimelineEdit | null;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onTrimPreviewRestore?: (positionMs: number) => void;
  onTrimPreviewStart?: () => void;
}) {
  const editGesture = useExportEditGesture();
  const [isActive, setIsActive] = useState(false);
  const [isRangeActive, setIsRangeActive] = useState(false);
  const [previewPosition, setPreviewPosition] = useState<number | null>(null);
  const [selectedSegmentId, setSelectedSegmentId] = useState<number | null>(
    null,
  );
  const [rangeSelection, setRangeSelection] =
    useState<TimelineRangeSelection | null>(null);
  const effectiveEdit = useMemo(
    () =>
      edit?.artifactId === artifactId
        ? edit
        : createRecordingTimelineEdit(artifactId),
    [artifactId, edit],
  );
  const retainedDuration = recordingTimelineRetainedDuration(effectiveEdit);
  const timelineDurationMs = totalDurationMs * retainedDuration;
  const effectiveSelectedSegmentId =
    selectedSegmentId !== null &&
    effectiveEdit.segments.some((segment) => segment.id === selectedSegmentId)
      ? selectedSegmentId
      : null;
  const positionRef = useRef(getPositionMs);
  positionRef.current = getPositionMs;
  const seekPlayerRef = useRef(seekPlayer);
  seekPlayerRef.current = seekPlayer;
  const previousEditRef = useRef(effectiveEdit);
  const parkedTrimOutputRef = useRef<number | null>(null);
  const parkedTrimSourceRef = useRef<number | null>(null);
  const snap = useCallback(
    (sourcePosition: number) =>
      snapRecordingTimelinePosition(
        sourcePosition,
        totalDurationMs,
        framesPerSecond,
      ),
    [framesPerSecond, totalDurationMs],
  );
  const snapOutput = useCallback(
    (outputPosition: number) =>
      recordingTimelineSourceToOutput(
        effectiveEdit,
        snap(recordingTimelineOutputToSource(effectiveEdit, outputPosition)),
      ),
    [effectiveEdit, snap],
  );

  const previewTrimFrame = useCallback(
    (sourcePosition: number, phase: "end" | "move" | "start") => {
      if (phase === "start") {
        const parkedSource =
          totalDurationMs > 0 ? positionRef.current() / totalDurationMs : 0;
        parkedTrimSourceRef.current = parkedSource;
        parkedTrimOutputRef.current = recordingTimelineSourceToOutput(
          effectiveEdit,
          parkedSource,
        );
        onTrimPreviewStart?.();
      }
      seekPlayerRef.current(sourcePosition * totalDurationMs, phase);
    },
    [effectiveEdit, onTrimPreviewStart, totalDurationMs],
  );
  const restoreTrimPreview = useCallback(() => {
    const sourcePosition = parkedTrimSourceRef.current;
    parkedTrimOutputRef.current = null;
    parkedTrimSourceRef.current = null;
    if (sourcePosition === null) return;
    const positionMs = sourcePosition * totalDurationMs;
    onTrimPreviewRestore?.(positionMs);
    seekPlayerRef.current(positionMs, "start");
    seekPlayerRef.current(positionMs, "end");
  }, [onTrimPreviewRestore, totalDurationMs]);
  const trim = useRecordingTimelineTrim({
    beginGesture: editGesture.beginGesture,
    edit: effectiveEdit,
    endGesture: editGesture.endGesture,
    framesPerSecond,
    onChange,
    onPreview: previewTrimFrame,
    onRestorePreview: restoreTrimPreview,
    selectSegment: setSelectedSegmentId,
    snap,
    totalDurationMs,
  });

  const seek = useCallback<SeekHandler>(
    (ratio, phase) => {
      const sourcePosition = recordingTimelineOutputToSource(
        effectiveEdit,
        ratio,
      );
      playhead.publish((ratio * timelineDurationMs) / 1_000, ratio);
      seekPlayerRef.current(sourcePosition * totalDurationMs, phase);
    },
    [effectiveEdit, playhead, timelineDurationMs, totalDurationMs],
  );

  useEffect(() => {
    const previous = previousEditRef.current;
    previousEditRef.current = effectiveEdit;
    if (
      previous === effectiveEdit ||
      previous.artifactId !== effectiveEdit.artifactId ||
      totalDurationMs <= 0
    )
      return;
    const currentSourceMs = positionRef.current();
    const parkedTrimOutput = parkedTrimOutputRef.current;
    if (trim.isActive() && parkedTrimOutput !== null) {
      parkedTrimSourceRef.current = recordingTimelineOutputToSource(
        effectiveEdit,
        parkedTrimOutput,
      );
      return;
    }
    const remapped = remapRecordingTimelinePosition(
      previous,
      effectiveEdit,
      currentSourceMs / totalDurationMs,
    );
    const nextTimelineDurationMs =
      totalDurationMs * recordingTimelineRetainedDuration(effectiveEdit);
    playhead.publish(
      (remapped.outputPosition * nextTimelineDurationMs) / 1_000,
      remapped.outputPosition,
    );
    const nextSourceMs = remapped.sourcePosition * totalDurationMs;
    if (Math.abs(nextSourceMs - currentSourceMs) < 0.5) return;
    seekPlayerRef.current(nextSourceMs, "start");
    seekPlayerRef.current(nextSourceMs, "end");
  }, [effectiveEdit, playhead, totalDurationMs, trim]);

  const cutSourceAt = useCallback(
    (sourcePosition: number) => {
      const next = cutKeyboardTimeline(effectiveEdit, snap(sourcePosition));
      if (next === effectiveEdit || !onChange) return;
      setSelectedSegmentId(null);
      editGesture.beginGesture();
      onChange(next);
      editGesture.endGesture();
    },
    [editGesture, effectiveEdit, onChange, snap],
  );
  const cutAt = useCallback(
    (outputPosition: number) => {
      cutSourceAt(
        recordingTimelineOutputToSource(effectiveEdit, outputPosition),
      );
    },
    [cutSourceAt, effectiveEdit],
  );
  const cutAtPlayhead = useCallback(() => {
    if (totalDurationMs <= 0) return;
    cutSourceAt(positionRef.current() / totalDurationMs);
  }, [cutSourceAt, totalDurationMs]);
  const deleteSelected = useCallback(() => {
    if (!onChange) return;
    const next = rangeSelection
      ? deleteRecordingTimelineRange(
          effectiveEdit,
          rangeSelection.start,
          rangeSelection.end,
        )
      : effectiveSelectedSegmentId === null
        ? effectiveEdit
        : deleteRecordingTimelineSegment(
            effectiveEdit,
            effectiveSelectedSegmentId,
          );
    if (next === effectiveEdit) return;
    setRangeSelection(null);
    setSelectedSegmentId(null);
    editGesture.beginGesture();
    onChange(next);
    editGesture.endGesture();
  }, [
    editGesture,
    effectiveEdit,
    effectiveSelectedSegmentId,
    onChange,
    rangeSelection,
  ]);
  const changeSegmentPlaybackRate = useCallback(
    (segmentId: number, playbackRate: number) => {
      if (!onChange) return;
      const next = setRecordingTimelineSegmentPlaybackRate(
        effectiveEdit,
        segmentId,
        playbackRate,
      );
      if (next === effectiveEdit) return;
      setRangeSelection(null);
      setSelectedSegmentId(segmentId);
      editGesture.beginGesture();
      onChange(next);
      editGesture.endGesture();
    },
    [editGesture, effectiveEdit, onChange],
  );
  const changeRangePlaybackRate = useCallback(
    (playbackRate: number) => {
      if (!onChange || !rangeSelection) return;
      const next = setRecordingTimelineRangePlaybackRate(effectiveEdit, {
        outputEnd: rangeSelection.end,
        outputStart: rangeSelection.start,
        playbackRate,
      });
      if (next === effectiveEdit) return;
      setRangeSelection(null);
      setSelectedSegmentId(null);
      editGesture.beginGesture();
      onChange(next);
      editGesture.endGesture();
    },
    [editGesture, effectiveEdit, onChange, rangeSelection],
  );
  const setActive = useCallback((active: boolean) => {
    setIsActive(active);
    if (active) {
      setIsRangeActive(false);
      setRangeSelection(null);
      setSelectedSegmentId(null);
    } else setPreviewPosition(null);
  }, []);
  const toggle = useCallback(() => {
    setIsActive((active) => {
      if (active) setPreviewPosition(null);
      else {
        setIsRangeActive(false);
        setRangeSelection(null);
        setSelectedSegmentId(null);
      }
      return !active;
    });
  }, []);
  const changeRangeActive = useCallback((active: boolean) => {
    setIsRangeActive(active);
    if (active) {
      setIsActive(false);
      setPreviewPosition(null);
      setSelectedSegmentId(null);
    } else setRangeSelection(null);
  }, []);
  const toggleRange = useCallback(() => {
    setIsRangeActive((active) => {
      if (active) setRangeSelection(null);
      else {
        setIsActive(false);
        setPreviewPosition(null);
        setSelectedSegmentId(null);
      }
      return !active;
    });
  }, []);
  const changeRangeSelection = useCallback(
    (anchor: number, focus: number) => {
      const start = snapOutput(Math.min(anchor, focus));
      const end = snapOutput(Math.max(anchor, focus));
      setRangeSelection(start === end ? null : { end, start });
    },
    [snapOutput],
  );
  const clearRangeSelection = useCallback(() => {
    setRangeSelection(null);
  }, []);
  const previewAt = useCallback(
    (sourcePosition: number) => {
      setPreviewPosition(snapOutput(sourcePosition));
    },
    [snapOutput],
  );
  const clearPreview = useCallback(() => {
    setPreviewPosition(null);
  }, []);
  const clearSelection = useCallback(() => {
    setSelectedSegmentId(null);
  }, []);
  const step = useCallback(
    (direction: -1 | 1, coarse: boolean) => {
      if (totalDurationMs <= 0) return;
      const outputPosition = recordingTimelineSourceToOutput(
        effectiveEdit,
        positionRef.current() / totalDurationMs,
      );
      const ratio =
        clamp(
          outputPosition * timelineDurationMs +
            direction * (coarse ? 1_000 : PREVIEW_FRAME_MS),
          0,
          timelineDurationMs,
        ) / timelineDurationMs;
      seek(ratio, "start");
      seek(ratio, "end");
    },
    [effectiveEdit, seek, timelineDurationMs, totalDurationMs],
  );

  useExportWindowShortcuts({
    onCutTimeline: shortcutsEnabled ? cutAtPlayhead : undefined,
    onDelete: shortcutsEnabled ? deleteSelected : undefined,
    onDeselect: shortcutsEnabled
      ? rangeSelection !== null
        ? clearRangeSelection
        : effectiveSelectedSegmentId !== null
          ? clearSelection
          : undefined
      : undefined,
    onToggleBladeTool: shortcutsEnabled ? toggle : undefined,
    onToggleRangeTool: shortcutsEnabled ? toggleRange : undefined,
  });

  return {
    blade: {
      beginTrim: trim.begin,
      clearPreview,
      clearRangeSelection,
      cutAt,
      edit: effectiveEdit,
      endTrim: trim.end,
      isActive,
      isRangeActive,
      previewAt,
      previewPosition,
      rangeSelection,
      selectSegment: setSelectedSegmentId,
      selectedSegmentId: effectiveSelectedSegmentId,
      setActive,
      setRangeActive: changeRangeActive,
      setRangePlaybackRate: changeRangePlaybackRate,
      setRangeSelection: changeRangeSelection,
      setSegmentPlaybackRate: changeSegmentPlaybackRate,
      snapPosition: snapOutput,
      updateTrim: trim.update,
    },
    seek,
    step,
    timelineDurationMs,
  };
}
