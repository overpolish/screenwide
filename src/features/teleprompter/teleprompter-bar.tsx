// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import {
  Maximize2,
  Mic,
  Minimize2,
  Pause,
  Play,
  RotateCcw,
  TrianglesCenterlineDashedVertical,
  X,
} from "lucide-react";

import { EditableTitle } from "../../composites/editable-title/editable-title";
import { t } from "../../i18n/i18n";
import {
  IconButton,
  IconToggleButton,
} from "../../primitives/button/icon-button";
import { NumberField } from "../../primitives/input-fields/number-field";
import { IslandParts } from "../../primitives/island/island-parts";
import { Text } from "../../primitives/text/text";

import {
  PACES,
  TEXT_SIZES,
  TeleprompterPreferences,
} from "./teleprompter-preferences";

export type TeleprompterBarProps = {
  isFullscreen: boolean;
  isPlaying: boolean;
  /** Whether the text moves at the steady pace. */
  isSteady: boolean;
  onClose: () => void;
  onFullscreenChange: (isFullscreen: boolean) => void;
  onPlayingChange: (isPlaying: boolean) => void;
  onPreferencesChange: (patch: Partial<TeleprompterPreferences>) => void;
  onRename: (title: string) => void;
  onRestart: () => void;
  preferences: TeleprompterPreferences;
  /** The script's name; empty while it is read. */
  title: string;
};

/**
 * The teleprompter's title bar, which moves the window: the script's name,
 * renamed where it is shown, then play and pause, back to the start,
 * following the voice, the steady pace while the text moves steadily, the
 * text's size, mirroring, filling the display and closing.
 */
export function TeleprompterBar({
  isFullscreen,
  isPlaying,
  isSteady,
  onClose,
  onFullscreenChange,
  onPlayingChange,
  onPreferencesChange,
  onRename,
  onRestart,
  preferences,
  title,
}: TeleprompterBarProps) {
  const { isMirrored, mode, textSize, wordsPerMinute } = preferences;
  return (
    <div
      className="flex min-h-control-height items-center gap-separate ps-inset pe-inset-sm"
      data-tauri-drag-region
    >
      <Text
        as="span"
        className="min-w-0 shrink truncate"
        data-tauri-drag-region
        variant="label"
      >
        {title ? (
          <EditableTitle
            label={t("teleprompter-script-name")}
            onChange={onRename}
            title={title}
          />
        ) : null}
      </Text>
      {/* The pace comes and goes with following the voice, and the
          controls beside it make room for it as it does. */}
      <IslandParts
        className="ms-auto flex shrink-0 items-center gap-inline"
        direction="row"
      >
        <IconToggleButton
          aria-label={
            isPlaying ? t("teleprompter-pause") : t("teleprompter-play")
          }
          isSelected={isPlaying}
          key="play"
          off={<Play className="fill-current" />}
          onChange={onPlayingChange}
        >
          <Pause className="fill-current" />
        </IconToggleButton>
        <IconButton
          aria-label={t("teleprompter-restart")}
          key="restart"
          onPress={onRestart}
        >
          <RotateCcw />
        </IconButton>
        <IconToggleButton
          aria-label={t("teleprompter-follow-voice")}
          isSelected={mode === "voice"}
          key="voice"
          onChange={(isOn) => {
            onPreferencesChange({ mode: isOn ? "voice" : "steady" });
          }}
        >
          <Mic />
        </IconToggleButton>
        {isSteady ? (
          <NumberField
            aria-label={t("teleprompter-pace")}
            key="pace"
            maxValue={PACES.max}
            minValue={PACES.min}
            onChange={(next) => {
              onPreferencesChange({ wordsPerMinute: next });
            }}
            reservedLength={3}
            rightSection={t("teleprompter-pace-unit")}
            step={PACES.step}
            value={wordsPerMinute}
          />
        ) : null}
        <NumberField
          aria-label={t("teleprompter-text-size")}
          key="size"
          maxValue={TEXT_SIZES.max}
          minValue={TEXT_SIZES.min}
          onChange={(next) => {
            onPreferencesChange({ textSize: next });
          }}
          reservedLength={2}
          rightSection="pt"
          step={TEXT_SIZES.step}
          value={textSize}
        />
        <IconToggleButton
          aria-label={t("teleprompter-mirror")}
          isSelected={isMirrored}
          key="mirror"
          onChange={(isOn) => {
            onPreferencesChange({ isMirrored: isOn });
          }}
        >
          <TrianglesCenterlineDashedVertical />
        </IconToggleButton>
        <IconToggleButton
          aria-label={t("teleprompter-fullscreen")}
          isSelected={isFullscreen}
          key="fullscreen"
          off={<Maximize2 />}
          onChange={onFullscreenChange}
        >
          <Minimize2 />
        </IconToggleButton>
        <IconButton
          aria-label={t("teleprompter-close")}
          key="close"
          onPress={onClose}
        >
          <X />
        </IconButton>
      </IslandParts>
    </div>
  );
}
