// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ClipboardCopy, Pause, Play } from "lucide-react";
import { memo } from "react";

import { Button } from "../../../components/base/button/button";
import { IconToggleButton } from "../../../components/base/button/icon-button";
import { ListBoxItem } from "../../../components/base/listbox-item/listbox-item";
import { Select } from "../../../components/base/select/select";
import { CheckOnClick } from "../../../components/shared/check-on-click/check-on-click";
import { formatDuration } from "../duration";

import { Playhead } from "./scrub-playhead";
import { ElapsedTime } from "./scrub-timeline";

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
  }: RecordingPlaybackControlsProps) {
    return (
      <div className="relative flex h-7 shrink-0 items-center justify-center gap-1.5 border-t border-muted/15 px-3">
        <IconToggleButton
          aria-keyshortcuts="P"
          aria-label={isPlaying ? "Pause preview" : "Play preview"}
          className="size-6 shrink-0"
          isSelected={isPlaying}
          off={<Play className="fill-current" size={14} />}
          onChange={(selected) => {
            if (selected) onPlay();
            else onPause();
          }}
          size="compact"
        >
          <Pause className="fill-current" size={14} />
        </IconToggleButton>
        <Select<(typeof PLAYBACK_RATES)[number]>
          aria-label="Preview speed"
          clearable={false}
          items={PLAYBACK_RATES}
          listBoxClassName="max-h-[inherit] min-w-20"
          onChange={(selection) => {
            const selected = PLAYBACK_RATES.find(
              (rate) => rate.id === selection,
            );
            if (selected) onPlaybackRateChange(selected.rate);
          }}
          popoverPlacement="bottom start"
          popoverShouldFlip={false}
          scrollShadow
          showFocus={false}
          size="compact"
          triggerClassName="h-6 gap-1 px-1.5 py-0"
          value={playbackRate.toString()}
        >
          {(rate) => (
            <ListBoxItem
              className="shrink-0"
              id={rate.id}
              textValue={rate.label}
            >
              {rate.label}
            </ListBoxItem>
          )}
        </Select>
        <span className="min-w-24 text-xs font-light text-content-fg tabular-nums">
          <ElapsedTime playhead={playhead} />
          <span className="text-muted"> / {formatDuration(durationMs)}</span>
        </span>
        {onCopyCurrentFrame ? (
          <CheckOnClick onPress={() => onCopyCurrentFrame()}>
            <Button
              aria-label="Copy current frame"
              className="absolute right-3"
              size="compact"
              variant="ghost"
            >
              <ClipboardCopy size={13} />
              Copy frame
            </Button>
          </CheckOnClick>
        ) : null}
      </div>
    );
  },
);
