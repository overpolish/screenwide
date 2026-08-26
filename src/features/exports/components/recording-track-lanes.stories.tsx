// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo, useRef, useState } from "react";

import {
  createRecordingTimelineEdit,
  cutRecordingTimeline,
  deleteRecordingTimelineSegment,
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  snapRecordingTimelinePosition,
} from "../recording-timeline-edit";
import { RecordingTrackId, RecordingVideoTrackId } from "../types";
import {
  ExportEditGestureContext,
  useExportEditHistory,
} from "../use-export-edit-history";
import { useExportWindowShortcuts } from "../use-export-window-shortcuts";

import { RecordingTrackLanes } from "./recording-track-lanes";
import { createPlayhead } from "./scrub-playhead";

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
    if (selectedSegmentId === null) return;
    const next = deleteRecordingTimelineSegment(
      timelineEdit,
      selectedSegmentId,
    );
    if (next === timelineEdit) return;
    setSelectedSegmentId(null);
    editGesture.beginGesture();
    setTimelineEdit(next);
    editGesture.endGesture();
  }, [editGesture, selectedSegmentId, timelineEdit]);
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
            cutAt,
            edit: timelineEdit,
            endTrim: () => undefined,
            isActive: isBladeActive,
            previewAt: (position) => {
              setPreviewPosition(snapOutput(position));
            },
            previewPosition,
            selectSegment: setSelectedSegmentId,
            selectedSegmentId,
            setActive: (active) => {
              setIsBladeActive(active);
              if (active) setSelectedSegmentId(null);
              if (!active) setPreviewPosition(null);
            },
            snapPosition: snapOutput,
            updateTrim: () => null,
          }}
          durationMs={timelineDurationMs}
          enabledTracks={enabledAudio}
          enabledVideoTracks={enabledVideo}
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
  title: "Features/Recording Timeline",
} satisfies Meta<typeof TimelinePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ZoomAndPan: Story = {};
