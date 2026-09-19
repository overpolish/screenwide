// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo, useRef, useState } from "react";

import {
  createRecordingTimelineEdit,
  cutRecordingTimeline,
  deleteRecordingTimelineRange,
  deleteRecordingTimelineSegment,
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  setRecordingTimelineSegmentPlaybackRate,
  snapRecordingTimelinePosition,
} from "../recording-timeline-edit";
import { setRecordingTimelineRangePlaybackRate } from "../recording-timeline-speed";
import { RecordingTrackId, RecordingVideoTrackId } from "../types";
import {
  EditorEditGestureContext,
  useEditorEditHistory,
} from "../use-editor-edit-history";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";

import { RecordingTrackLanes } from "./recording-track-lanes";
import {
  STORY_ANNOTATION_CLIPS,
  STORY_DURATION_MS,
  STORY_FRAMES_PER_SECOND,
  STORY_KEYBOARD_ITEMS,
  STORY_LAYOUT,
  STORY_THUMBNAILS,
} from "./recording-track-lanes-preview-fixtures";
import { createPlayhead } from "./scrub-playhead";
import { selectTimelineItem } from "./timeline-item-selection";

/**
 * The timeline lanes wired to real edit state, shared by the lanes story and
 * the resizable band story so neither has to hand-roll a shell around them.
 * `audioTrackCount` grows the band past a short viewport, which is what makes
 * the scrolling and its overflow shadows worth looking at.
 */
export function RecordingTrackLanesPreview({
  audioTrackCount = 1,
}: {
  audioTrackCount?: number;
}) {
  const audioTracks = useMemo(
    () =>
      Array.from({ length: audioTrackCount }, (_, track) => ({
        kind: "system-audio" as const,
        label: track === 0 ? "System audio" : `Audio ${(track + 1).toString()}`,
        streamIndex: track,
        waveform: Array.from(
          { length: 240 },
          (_, index) =>
            0.15 + Math.abs(Math.sin(index * 0.19 + track * 0.7)) * 0.75,
        ),
      })),
    [audioTrackCount],
  );

  const playhead = useMemo(() => createPlayhead(), []);
  const [enabledAudio, setEnabledAudio] = useState(() => new Set([0]));
  const [enabledVideo, setEnabledVideo] = useState<Set<RecordingVideoTrackId>>(
    () => new Set(["primary"]),
  );
  const [annotationClips, setAnnotationClips] = useState(
    STORY_ANNOTATION_CLIPS,
  );
  const [selectedAnnotationId, setSelectedAnnotationId] = useState<
    string | null
  >(null);
  const [isBladeActive, setIsBladeActive] = useState(false);
  const [previewPosition, setPreviewPosition] = useState<number | null>(null);
  const [isRangeActive, setIsRangeActive] = useState(false);
  const [isSnapActive, setIsSnapActive] = useState(true);
  const [snapGuidePosition, setSnapGuidePosition] = useState<number | null>(
    null,
  );
  const [selectedKeyboardItems, setSelectedKeyboardItems] = useState(
    () => new Set<string>(),
  );
  const [rangeSelection, setRangeSelection] = useState<{
    end: number;
    start: number;
  } | null>(null);
  const [selectedSegmentId, setSelectedSegmentId] = useState<number | null>(
    null,
  );
  const [selectedTrack, setSelectedTrack] = useState<RecordingTrackId | null>(
    "primary",
  );
  const [timelineEdit, setTimelineEdit] = useState(() =>
    createRecordingTimelineEdit(1),
  );
  const playheadRatioRef = useRef(0);
  const editGesture = useEditorEditHistory({
    apply: setTimelineEdit,
    resetKey: 1,
    state: timelineEdit,
  });
  const timelineDurationMs =
    STORY_DURATION_MS * recordingTimelineRetainedDuration(timelineEdit);
  const snapOutput = useCallback(
    (outputPosition: number) =>
      recordingTimelineSourceToOutput(
        timelineEdit,
        snapRecordingTimelinePosition(
          recordingTimelineOutputToSource(timelineEdit, outputPosition),
          STORY_DURATION_MS,
          STORY_FRAMES_PER_SECOND,
        ),
      ),
    [timelineEdit],
  );
  const cutAt = useCallback(
    (outputPosition: number) => {
      const next = cutRecordingTimeline(
        timelineEdit,
        snapRecordingTimelinePosition(
          recordingTimelineOutputToSource(timelineEdit, outputPosition),
          STORY_DURATION_MS,
          STORY_FRAMES_PER_SECOND,
        ),
      );
      if (next === timelineEdit) return;
      setSelectedSegmentId(null);
      editGesture.beginGesture();
      setTimelineEdit(next);
      editGesture.endGesture();
    },
    [editGesture, timelineEdit],
  );
  const deleteSelected = useCallback(() => {
    const next = rangeSelection
      ? deleteRecordingTimelineRange(
          timelineEdit,
          rangeSelection.start,
          rangeSelection.end,
        )
      : selectedSegmentId === null
        ? timelineEdit
        : deleteRecordingTimelineSegment(timelineEdit, selectedSegmentId);
    if (next === timelineEdit) return;
    setRangeSelection(null);
    setSelectedSegmentId(null);
    editGesture.beginGesture();
    setTimelineEdit(next);
    editGesture.endGesture();
  }, [editGesture, rangeSelection, selectedSegmentId, timelineEdit]);
  useEditorWindowShortcuts({
    onCutTimeline: () => {
      cutAt(playheadRatioRef.current);
    },
    onDelete: deleteSelected,
    onDeselect:
      selectedSegmentId === null
        ? undefined
        : () => {
            setSelectedSegmentId(null);
          },
    onToggleBladeTool: () => {
      setIsBladeActive((active) => !active);
    },
    onToggleSnap: () => {
      setIsSnapActive((active) => !active);
    },
  });

  return (
    <EditorEditGestureContext value={editGesture}>
      <RecordingTrackLanes
        adjustedKeyboardFragmentIds={new Set()}
        annotationClips={annotationClips}
        audioTracks={audioTracks}
        blade={{
          beginTrim: () => undefined,
          clearPreview: () => {
            setPreviewPosition(null);
          },
          clearRangeSelection: () => {
            setRangeSelection(null);
          },
          cutAt,
          edit: timelineEdit,
          endTrim: () => undefined,
          isActive: isBladeActive,
          isRangeActive,
          isSnapActive,
          previewAt: (position) => {
            setPreviewPosition(snapOutput(position));
          },
          previewPosition,
          rangeSelection,
          selectSegment: setSelectedSegmentId,
          selectedSegmentId,
          setActive: (active) => {
            setIsBladeActive(active);
            if (active) setIsRangeActive(false);
            if (active) setSelectedSegmentId(null);
            if (!active) setPreviewPosition(null);
          },
          setRangeActive: (active) => {
            setIsRangeActive(active);
            if (active) {
              setIsBladeActive(false);
              setSelectedSegmentId(null);
            } else setRangeSelection(null);
          },
          setRangePlaybackRate: (playbackRate) => {
            if (!rangeSelection) return;
            setTimelineEdit((current) =>
              setRecordingTimelineRangePlaybackRate(current, {
                outputEnd: rangeSelection.end,
                outputStart: rangeSelection.start,
                playbackRate,
              }),
            );
            setRangeSelection(null);
          },
          setRangeSelection: (anchor, focus) => {
            const start = snapOutput(Math.min(anchor, focus));
            const end = snapOutput(Math.max(anchor, focus));
            setRangeSelection(start === end ? null : { end, start });
          },
          setSegmentPlaybackRate: (segmentId, playbackRate) => {
            setTimelineEdit((current) =>
              setRecordingTimelineSegmentPlaybackRate(
                current,
                segmentId,
                playbackRate,
              ),
            );
          },
          setSnapActive: setIsSnapActive,
          setSnapGuidePosition,
          snapGuidePosition,
          snapPosition: snapOutput,
          updateTrim: () => null,
        }}
        durationMs={timelineDurationMs}
        enabledTracks={enabledAudio}
        enabledVideoTracks={enabledVideo}
        hiddenKeyboardFragmentIds={new Set()}
        hiddenKeyboardItemIds={new Set()}
        keyboardItems={STORY_KEYBOARD_ITEMS}
        keyboardSelection={{
          ids: selectedKeyboardItems,
          onClear: () => {
            setSelectedKeyboardItems(new Set());
          },
          onSelect: (itemId, toggle) => {
            setSelectedKeyboardItems((current) =>
              selectTimelineItem(current, itemId, toggle),
            );
          },
        }}
        layout={STORY_LAYOUT}
        onAnnotationsChange={setAnnotationClips}
        onAnnotationSelect={setSelectedAnnotationId}
        onEnabledTracksChange={setEnabledAudio}
        onEnabledVideoTracksChange={setEnabledVideo}
        onSeek={(ratio) => {
          playheadRatioRef.current = ratio;
          playhead.publish((ratio * timelineDurationMs) / 1_000, ratio);
        }}
        onSelectedTrackChange={setSelectedTrack}
        playhead={playhead}
        selectedAnnotationId={selectedAnnotationId}
        selectedTrack={selectedTrack}
        sourceDurationMs={STORY_DURATION_MS}
        thumbnails={STORY_THUMBNAILS}
        videoTrackOrder={["primary"]}
        volumes={new Map()}
      />
    </EditorEditGestureContext>
  );
}
