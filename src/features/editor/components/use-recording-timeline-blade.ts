// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef } from "react";

import { PREVIEW_FRAME_MS } from "../duration";
import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  remapRecordingTimelinePosition,
  snapRecordingTimelinePosition,
} from "../recording-timeline-edit";
import { useEditorEditGesture } from "../use-editor-edit-history";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";

import { clamp, Playhead } from "./scrub-playhead";
import { SeekHandler } from "./timeline-seek";
import { useRecordingTimelineTrim } from "./use-recording-timeline-trim";
import { useTimelineBladeCommands } from "./use-timeline-blade-commands";
import { useTimelineBladeSelection } from "./use-timeline-blade-selection";
import { useTimelineSeek } from "./use-timeline-seek";

export function useRecordingTimelineBlade({
  artifactId,
  edit,
  framesPerSecond,
  getPositionMs,
  onChange,
  onTrimPreviewRestore,
  onTrimPreviewStart,
  ownsDelete,
  playhead,
  seekPlayer,
  shortcutsEnabled,
  totalDurationMs,
}: {
  artifactId: number;
  framesPerSecond: number | null;
  getPositionMs: () => number;
  /** False while an annotation is what Delete would take: the segment
   * selection then keeps out of its way. Every other shortcut stays live. */
  ownsDelete: boolean;
  playhead: Playhead;
  seekPlayer: SeekHandler;
  shortcutsEnabled: boolean;
  totalDurationMs: number;
  edit?: RecordingTimelineEdit | null;
  onChange?: (edit: RecordingTimelineEdit) => void;
  onTrimPreviewRestore?: (positionMs: number) => void;
  onTrimPreviewStart?: () => void;
}) {
  const editGesture = useEditorEditGesture();
  const effectiveEdit = useMemo(
    () =>
      edit?.artifactId === artifactId
        ? edit
        : createRecordingTimelineEdit(artifactId),
    [artifactId, edit],
  );
  const retainedDuration = recordingTimelineRetainedDuration(effectiveEdit);
  const timelineDurationMs = totalDurationMs * retainedDuration;
  const positionRef = useRef(getPositionMs);
  positionRef.current = getPositionMs;
  const seekPlayerRef = useRef(seekPlayer);
  seekPlayerRef.current = seekPlayer;
  const previousEditRef = useRef(effectiveEdit);
  const parkedTrimOutputRef = useRef<number | null>(null);
  // Logged only when it flips, so hovering the lane does not flood the console.
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

  const selection = useTimelineBladeSelection({
    edit: effectiveEdit,
    snap,
    snapOutput,
  });
  const commands = useTimelineBladeCommands({
    clearRangeSelection: selection.clearRangeSelection,
    edit: effectiveEdit,
    onChange,
    positionRef,
    rangeSelection: selection.rangeSelection,
    selectSegment: selection.selectSegment,
    selectedSegmentId: selection.selectedSegmentId,
    snap,
    totalDurationMs,
  });

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
    selectSegment: selection.selectSegment,
    snap,
    totalDurationMs,
  });

  const seek = useTimelineSeek({
    durationMs: totalDurationMs,
    edit: effectiveEdit,
    player: seekPlayerRef,
    playhead,
  });

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

  useEditorWindowShortcuts({
    onCutTimeline: shortcutsEnabled ? commands.cutAtPlayhead : undefined,
    onDelete:
      shortcutsEnabled && ownsDelete ? commands.deleteSelected : undefined,
    onDeselect: shortcutsEnabled
      ? selection.rangeSelection !== null
        ? selection.clearRangeSelection
        : selection.selectedSegmentId !== null
          ? selection.clearSelection
          : undefined
      : undefined,
    onToggleBladeTool: shortcutsEnabled ? selection.toggle : undefined,
    onToggleRangeTool: shortcutsEnabled ? selection.toggleRange : undefined,
    onToggleSnap: shortcutsEnabled ? selection.toggleSnap : undefined,
  });

  return {
    blade: {
      beginTrim: trim.begin,
      clearPreview: selection.clearPreview,
      clearRangeSelection: selection.clearRangeSelection,
      cutAt: commands.cutAt,
      edit: effectiveEdit,
      endTrim: trim.end,
      isActive: selection.isActive,
      isRangeActive: selection.isRangeActive,
      isSnapActive: selection.isSnapActive,
      previewAt: selection.previewAt,
      previewPosition: selection.previewPosition,
      rangeSelection: selection.rangeSelection,
      selectSegment: selection.selectSegment,
      selectedSegmentId: selection.selectedSegmentId,
      setActive: selection.setActive,
      setRangeActive: selection.setRangeActive,
      setRangePlaybackRate: commands.changeRangePlaybackRate,
      setRangeSelection: selection.setRangeSelection,
      setSegmentPlaybackRate: commands.changeSegmentPlaybackRate,
      setSnapActive: selection.setSnapActive,
      setSnapGuidePosition: selection.setSnapGuidePosition,
      snapGuidePosition: selection.snapGuidePosition,
      snapPosition: snapOutput,
      updateTrim: trim.update,
    },
    seek,
    step,
    timelineDurationMs,
  };
}
