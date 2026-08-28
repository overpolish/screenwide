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
  ExportEditGestureContext,
  useExportEditHistory,
} from "../use-export-edit-history";
import { useExportWindowShortcuts } from "../use-export-window-shortcuts";

import { RecordingTrackLanes } from "./recording-track-lanes";
import { createPlayhead } from "./scrub-playhead";
import { selectTimelineItem } from "./timeline-item-selection";

import type { Meta, StoryObj } from "@storybook/react-vite";

const STORY_DURATION_MS = 120_000;
const STORY_FRAMES_PER_SECOND = 60_000 / 1_001;

function TimelinePreview() {
  const playhead = useMemo(() => createPlayhead(), []);
  const [enabledAudio, setEnabledAudio] = useState(() => new Set([0]));
  const [enabledVideo, setEnabledVideo] = useState<Set<RecordingVideoTrackId>>(
    () => new Set(["primary"]),
  );
  const [isBladeActive, setIsBladeActive] = useState(false);
  const [previewPosition, setPreviewPosition] = useState<number | null>(null);
  const [isRangeActive, setIsRangeActive] = useState(false);
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
  const editGesture = useExportEditHistory({
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
  useExportWindowShortcuts({
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
  });

  return (
    <ExportEditGestureContext value={editGesture}>
      <div className="w-[760px] bg-content text-content-fg">
        <RecordingTrackLanes
          adjustedKeyboardFragmentIds={new Set()}
          audioTracks={[
            {
              kind: "system-audio",
              label: "System audio",
              streamIndex: 0,
              waveform: Array.from(
                { length: 240 },
                (_, index) => 0.15 + Math.abs(Math.sin(index * 0.19)) * 0.75,
              ),
            },
          ]}
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
            snapPosition: snapOutput,
            updateTrim: () => null,
          }}
          durationMs={timelineDurationMs}
          enabledTracks={enabledAudio}
          enabledVideoTracks={enabledVideo}
          hiddenKeyboardFragmentIds={new Set()}
          hiddenKeyboardItemIds={new Set()}
          keyboardItems={[
            { endMs: 11_800, id: 0, label: "⌘ C", startMs: 10_000 },
            { endMs: 29_600, id: 1, label: "⌘ V", startMs: 28_000 },
            { endMs: 74_900, id: 2, label: "⇧ ⌘ 4", startMs: 72_000 },
          ]}
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
          layout={{
            height: 1080,
            panes: [
              {
                height: 1080,
                kind: "screen",
                sourceHeight: 1080,
                sourceWidth: 1920,
                width: 1920,
                x: 0,
                y: 0,
              },
            ],
            width: 1920,
          }}
          onEnabledTracksChange={setEnabledAudio}
          onEnabledVideoTracksChange={setEnabledVideo}
          onSeek={(ratio) => {
            playheadRatioRef.current = ratio;
            playhead.publish((ratio * timelineDurationMs) / 1_000, ratio);
          }}
          onSelectedTrackChange={setSelectedTrack}
          playhead={playhead}
          selectedTrack={selectedTrack}
          sourceDurationMs={STORY_DURATION_MS}
          thumbnails={{
            camera: [],
            primary: Array.from({ length: 24 }, (_, index) => ({
              id: `primary-${index.toString()}`,
              url: null,
            })),
          }}
          videoTrackOrder={["primary"]}
          volumes={new Map()}
        />
      </div>
    </ExportEditGestureContext>
  );
}

const meta = {
  component: TimelinePreview,
  parameters: { layout: "centered" },
  title: "Legacy/Recording Timeline",
} satisfies Meta<typeof TimelinePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ZoomAndPan: Story = {};
