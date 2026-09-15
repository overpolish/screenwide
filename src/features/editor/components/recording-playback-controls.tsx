// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Images, Pause, Play } from "lucide-react";
import { memo, ReactNode } from "react";

import {
  IconButton,
  IconToggleButton,
} from "../../../components/base/button/icon-button";
import { CheckOnClick } from "../../../components/shared/check-on-click/check-on-click";
import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";
import { PopupSelect } from "../../popup-panel/popup-select";
import { formatDuration } from "../duration";

import { Playhead } from "./scrub-playhead";
import { ElapsedTime } from "./scrub-timeline";
import { useRegisterTimelinePlaybackRow } from "./timeline-band-playback-row";

type RecordingPlaybackControlsProps = {
  durationMs: number;
  isPlaying: boolean;
  onPause: () => void;
  onPlay: () => void;
  onPlaybackRateChange: (rate: number) => void;
  playbackRate: number;
  playhead: Playhead;
  // Returning a promise makes the copy button await the copy before it checks.
  onCopyCurrentFrame?: () => Promise<unknown> | undefined;
  /** How close the picture is drawn, at the left of the row. */
  zoomControl?: ReactNode;
};

const PLAYBACK_RATES = [0.5, 0.75, 1, 1.25, 1.5, 2].map((rate) => ({
  id: rate.toString(),
  label: `${rate.toString()}×`,
  rate,
}));

/**
 * Memoized: the playhead publishes its own time through a subscription, so
 * nothing here changes while an output draft updates at pointer rate.
 */
export const RecordingPlaybackControls = memo(
  function RecordingPlaybackControls({
    durationMs,
    isPlaying,
    onCopyCurrentFrame,
    onPause,
    onPlay,
    onPlaybackRateChange,
    playbackRate,
    playhead,
    zoomControl,
  }: RecordingPlaybackControlsProps) {
    // Inside the resizable band this row stands above the scroller, and
    // reports its height so the band can count it towards its own.
    const registerPlaybackRow = useRegisterTimelinePlaybackRow();
    return (
      // The band's first row: standard controls inside the window inset, with
      // the transport on the window's centre line whatever stands either side.
      <div
        // No fill of its own: the band already paints one, and a second copy
        // tints this strip twice and breaks the band into tones.
        className="relative flex min-h-control-height shrink-0 items-center justify-center gap-control px-window-inset py-control-inset"
        ref={registerPlaybackRow}
      >
        {zoomControl ? (
          <div className="absolute left-window-inset flex items-center gap-section">
            {zoomControl}
          </div>
        ) : null}
        <PopupSelect
          className="w-20"
          id="preview-speed"
          items={PLAYBACK_RATES}
          label="Preview speed"
          minimumListWidth={120}
          onSelectionChange={(item) => {
            const selected = PLAYBACK_RATES.find((rate) => rate.id === item.id);
            if (selected) onPlaybackRateChange(selected.rate);
          }}
          placeholder="Speed"
          selectedId={playbackRate.toString()}
        />
        <IconToggleButton
          aria-keyshortcuts="P"
          aria-label={isPlaying ? "Pause preview" : "Play preview"}
          className="shrink-0"
          isSelected={isPlaying}
          off={<Play className="fill-current" />}
          onChange={(selected) => {
            if (selected) onPlay();
            else onPause();
          }}
        >
          <Pause className="fill-current" />
        </IconToggleButton>
        <span className="min-w-24 text-body text-content-fg tabular-nums">
          <ElapsedTime playhead={playhead} />
          <span className="text-content-fg-secondary">
            {" "}
            / {formatDuration(durationMs)}
          </span>
        </span>
        {onCopyCurrentFrame ? (
          <div className="absolute right-window-inset flex items-center gap-section">
            <NativeTooltipTrigger tooltip="Copy Frame">
              <CheckOnClick onPress={() => onCopyCurrentFrame()}>
                <IconButton aria-label="Copy current frame">
                  <Images />
                </IconButton>
              </CheckOnClick>
            </NativeTooltipTrigger>
          </div>
        ) : null}
      </div>
    );
  },
);
