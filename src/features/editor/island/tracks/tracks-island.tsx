// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Layers } from "lucide-react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { Switch } from "../../../../primitives/switch/switch";

import type { EditorIslandContent } from "../editor-island";

// The tracks the recording holds, each with a switch that keeps it in the
// export and the preview or leaves it out. Something has to be left to
// export, so the last track still in cannot be switched off. The island
// stands at the picture's top end corner, as every editor island does.

type TracksIslandTrack = {
  id: string;
  isEnabled: boolean;
  /** Its own name, such as Screen or Microphone. */
  label: string;
};

export type TracksIslandProps = {
  onEnabledChange: (trackId: string, isEnabled: boolean) => void;
  /** Video first, then audio, as the timeline listed them. */
  tracks: TracksIslandTrack[];
};

export function tracksIsland({
  onEnabledChange,
  tracks,
}: TracksIslandProps): EditorIslandContent {
  const enabledCount = tracks.filter(({ isEnabled }) => isEnabled).length;
  return {
    key: "tracks",
    section: (
      <div className="flex flex-col gap-inline">
        {tracks.map((track) => (
          <ControlRow key={track.id} title={track.label}>
            {(controlProps) => (
              <Switch
                {...controlProps}
                isDisabled={track.isEnabled && enabledCount === 1}
                isSelected={track.isEnabled}
                onChange={(isEnabled) => {
                  onEnabledChange(track.id, isEnabled);
                }}
              />
            )}
          </ControlRow>
        ))}
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<Layers aria-hidden className="size-icon" />}
        title={t("editor-island-tracks")}
      />
    ),
  };
}
