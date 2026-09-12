// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useRef, useState } from "react";

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";

import { PreviewZoomField } from "./preview-readouts";
import { RecordingPlaybackControls } from "./recording-playback-controls";
import { createPlayhead } from "./scrub-playhead";

import type { Meta, StoryObj } from "@storybook/react-vite";

const STORY_DURATION_MS = 96_000;

/**
 * The band's closing row on its own fill, with the zoom field it carries in
 * the editor. The playhead runs off a timer so the readout reads as it does
 * during playback.
 */
function PlaybackBarPreview() {
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
    <div className="bg-fill-quaternary">
      <RecordingPlaybackControls
        durationMs={STORY_DURATION_MS}
        isPlaying={isPlaying}
        onCopyCurrentFrame={() => Promise.resolve()}
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
    </div>
  );
}

const meta = {
  component: PlaybackBarPreview,
  parameters: { layout: "centered" },
  render: (_args, { viewMode }) => (
    <FeatureStoryStage viewMode={viewMode} width={720}>
      <PlaybackBarPreview />
    </FeatureStoryStage>
  ),
  title: "Features/Editor Playback Bar",
} satisfies Meta<typeof PlaybackBarPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Transport: Story = {};
