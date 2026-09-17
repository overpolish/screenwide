// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback } from "react";

import { cutKeyboardTimeline } from "../recording-keyboard-timeline-cut";
import {
  deleteRecordingTimelineRange,
  deleteRecordingTimelineSegment,
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  setRecordingTimelineSegmentPlaybackRate,
} from "../recording-timeline-edit";
import { setRecordingTimelineRangePlaybackRate } from "../recording-timeline-speed";
import { useEditorEditGesture } from "../use-editor-edit-history";

import { TimelineRangeSelection } from "./timeline-blade";

/**
 * The edits the blade band commits: a cut, a deletion, and a playback rate,
 * each one history step of its own. Every command drops the selection it
 * consumed, since the segment ids it named no longer describe the timeline.
 */
export function useTimelineBladeCommands({
  clearRangeSelection,
  edit,
  onChange,
  positionRef,
  rangeSelection,
  selectSegment,
  selectedSegmentId,
  snap,
  totalDurationMs,
}: {
  clearRangeSelection: () => void;
  edit: RecordingTimelineEdit;
  positionRef: RefObject<() => number>;
  rangeSelection: TimelineRangeSelection | null;
  selectSegment: (segmentId: number | null) => void;
  selectedSegmentId: number | null;
  snap: (sourcePosition: number) => number;
  totalDurationMs: number;
  onChange?: (edit: RecordingTimelineEdit) => void;
}) {
  const editGesture = useEditorEditGesture();
  const cutSourceAt = useCallback(
    (sourcePosition: number) => {
      const snapped = snap(sourcePosition);
      const next = cutKeyboardTimeline(edit, snapped);
      if (next === edit || !onChange) return;
      selectSegment(null);
      editGesture.beginGesture();
      onChange(next);
      editGesture.endGesture();
    },
    [edit, editGesture, onChange, selectSegment, snap],
  );
  const cutAt = useCallback(
    (outputPosition: number) => {
      cutSourceAt(recordingTimelineOutputToSource(edit, outputPosition));
    },
    [cutSourceAt, edit],
  );
  const cutAtPlayhead = useCallback(() => {
    if (totalDurationMs <= 0) return;
    cutSourceAt(positionRef.current() / totalDurationMs);
  }, [cutSourceAt, positionRef, totalDurationMs]);
  const deleteSelected = useCallback(() => {
    if (!onChange) return;
    const next = rangeSelection
      ? deleteRecordingTimelineRange(
          edit,
          rangeSelection.start,
          rangeSelection.end,
        )
      : selectedSegmentId === null
        ? edit
        : deleteRecordingTimelineSegment(edit, selectedSegmentId);
    if (next === edit) return;
    clearRangeSelection();
    selectSegment(null);
    editGesture.beginGesture();
    onChange(next);
    editGesture.endGesture();
  }, [
    clearRangeSelection,
    edit,
    editGesture,
    onChange,
    rangeSelection,
    selectSegment,
    selectedSegmentId,
  ]);
  const changeSegmentPlaybackRate = useCallback(
    (segmentId: number, playbackRate: number) => {
      if (!onChange) return;
      const next = setRecordingTimelineSegmentPlaybackRate(
        edit,
        segmentId,
        playbackRate,
      );
      if (next === edit) return;
      clearRangeSelection();
      selectSegment(segmentId);
      editGesture.beginGesture();
      onChange(next);
      editGesture.endGesture();
    },
    [clearRangeSelection, edit, editGesture, onChange, selectSegment],
  );
  const changeRangePlaybackRate = useCallback(
    (playbackRate: number) => {
      if (!onChange || !rangeSelection) return;
      const next = setRecordingTimelineRangePlaybackRate(edit, {
        outputEnd: rangeSelection.end,
        outputStart: rangeSelection.start,
        playbackRate,
      });
      if (next === edit) return;
      clearRangeSelection();
      selectSegment(null);
      editGesture.beginGesture();
      onChange(next);
      editGesture.endGesture();
    },
    [
      clearRangeSelection,
      edit,
      editGesture,
      onChange,
      rangeSelection,
      selectSegment,
    ],
  );

  return {
    changeRangePlaybackRate,
    changeSegmentPlaybackRate,
    cutAt,
    cutAtPlayhead,
    deleteSelected,
  };
}
