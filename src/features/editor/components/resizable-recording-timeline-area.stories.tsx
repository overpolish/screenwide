// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useRef, useState } from "react";

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";

import { PreviewZoomField } from "./preview-readouts";
import { RecordingPlaybackControls } from "./recording-playback-controls";
import { RecordingTrackLanesPreview } from "./recording-track-lanes-preview";
import { ResizableRecordingTimelineArea } from "./resizable-recording-timeline-area";
import { createPlayhead } from "./scrub-playhead";

import type { Meta, StoryObj } from "@storybook/react-vite";

const STORY_DURATION_MS = 96_000;

/**
 * The band as the editor assembles it: the real transport row above the real
 * lanes. Drag the divider down to the stop - it never hides the transport, the
 * ruler and two lane rows - and only the lane rows scroll: the transport row,
 * the ruler and the meter all stand still, so the overflow shadow marks
 * exactly where the rows are cut off. Double click the divider to refit.
 */
function BandPreview({ audioTrackCount }: { audioTrackCount: number }) {
  const playhead = useMemo(() => createPlayhead(), []);
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackRate, setPlaybackRate] = useState(1);
  const [zoomPercent, setZoomPercent] = useState(100);
  const positionMsRef = useRef(12_000);

  useEffect(() => {
    playhead.publish(
      positionMsRef.current / 1_000,
      positionMsRef.current / STORY_DURATION_MS,
    );
    if (!isPlaying) return;
    const interval = setInterval(() => {
      positionMsRef.current =
        (positionMsRef.current + 100 * playbackRate) % STORY_DURATION_MS;
      playhead.publish(
        positionMsRef.current / 1_000,
        positionMsRef.current / STORY_DURATION_MS,
      );
    }, 100);
    return () => {
      clearInterval(interval);
    };
  }, [isPlaying, playbackRate, playhead]);

  return (
    <div className="flex h-full min-h-0 flex-col">
      {/* Stands in for the preview picture the band is dragged against. */}
      <div className="min-h-0 flex-1 bg-fill-tertiary" />
      <ResizableRecordingTimelineArea
        artifactId={1}
        header={
          <RecordingPlaybackControls
            durationMs={STORY_DURATION_MS}
            isPlaying={isPlaying}
            onPause={() => {
              setIsPlaying(false);
            }}
            onPlay={() => {
              setIsPlaying(true);
            }}
            onPlaybackRateChange={setPlaybackRate}
            playbackRate={playbackRate}
            playhead={playhead}
            zoomControl={
              <PreviewZoomField
                onChange={setZoomPercent}
                zoomPercent={zoomPercent}
              />
            }
          />
        }
      >
        <RecordingTrackLanesPreview audioTrackCount={audioTrackCount} />
      </ResizableRecordingTimelineArea>
    </div>
  );
}

const meta = {
  component: BandPreview,
  parameters: { layout: "centered" },
  render: ({ audioTrackCount }, { viewMode }) => (
    <FeatureStoryStage height={440} viewMode={viewMode} width={760}>
      <BandPreview audioTrackCount={audioTrackCount} />
    </FeatureStoryStage>
  ),
  title: "Features/Editor/Resizable Recording Timeline Area",
} satisfies Meta<typeof BandPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Fits its content on mount: nothing scrolls, so neither edge is shadowed. */
export const Default: Story = { args: { audioTrackCount: 1 } };

/**
 * More lanes than the band can ever show: the transport row, the ruler and the
 * meter all stay put, the lane rows scroll under them behind an overflow
 * shadow that spans the label gutter, the playhead runs the full height of the
 * ruler and the visible rows, and the drag stop keeps two lane rows on screen
 * at the smallest size.
 */
export const Scrolling: Story = { args: { audioTrackCount: 6 } };
