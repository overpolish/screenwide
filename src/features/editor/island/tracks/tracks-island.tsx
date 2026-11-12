// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Layers } from "lucide-react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { Switch } from "../../../../primitives/switch/switch";

import type { EditorIslandContent } from "../editor-island";

// The tracks the recording holds, each with a switch that keeps it in the
// export and the preview or leaves it out, then the songs and the sound
// effects laid under it, where there are any. Something recorded has to be
// left to export, so the last recorded track still in cannot be switched
// off. The island stands at the picture's top end corner, as every editor
// island does. The section is a list of switches and nothing else, so no
// fill holds the section's edge: the switches come in to the text column
// the names start on, and the list sits evenly between the island's sides.

type TracksIslandTrack = {
  id: string;
  isEnabled: boolean;
  /** Its own name, such as Screen or Microphone. */
  label: string;
  /** Laid under the recording rather than recorded. */
  isLaidUnder?: boolean;
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
  const enabledCount = tracks.filter(
    ({ isEnabled, isLaidUnder }) => isEnabled && !isLaidUnder,
  ).length;
  return {
    key: "tracks",
    section: (
      <div className="flex flex-col gap-inline pe-inset-sm">
        {tracks.map((track) => (
          <ControlRow key={track.id} title={track.label}>
            {(controlProps) => (
              <Switch
                {...controlProps}
                isDisabled={
                  track.isEnabled && !track.isLaidUnder && enabledCount === 1
                }
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
