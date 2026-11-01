// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cursorIsland } from "../cursor/cursor-island";
import { keyboardIsland } from "../keyboard/keyboard-island";
import { soundIsland, type SoundTool } from "../sound/sound-island";
import { tracksIsland } from "../tracks/tracks-island";

import type { ToolPanelPatch } from "../../tool-panels/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tool-panels/tool-panel-store";
import type { EditorIslandContent } from "../editor-island";

// The islands that hold something of the whole recording at once: how the
// cursor and the shortcuts pressed are drawn, every track's sound, and
// which tracks it plays and exports. The title bar offers each mode only
// where the recording has what it holds (`editorModeGroups`).

type Inputs = {
  change: (values: ToolPanelPatch) => void;
  snapshot: ToolPanelSnapshot;
};

export function cursorIslandContent({
  change,
  snapshot,
}: Inputs): EditorIslandContent {
  return cursorIsland({
    onChange: (cursorEffects) => {
      change({ cursorEffects });
    },
    settings: snapshot.cursorEffects,
  });
}

export function keyboardIslandContent({
  change,
  snapshot,
}: Inputs): EditorIslandContent {
  return keyboardIsland({
    canRestoreShortcuts: snapshot.canRestoreShortcuts,
    chosen: snapshot.chosenShortcuts,
    onChange: (keyboardEffects) => {
      change({ keyboardEffects });
    },
    onDeleteChosen: () => {
      change({ deleteChosenShortcuts: true });
    },
    onRestoreShortcuts: () => {
      change({ restoreShortcuts: true });
    },
    settings: snapshot.keyboardEffects,
  });
}

/** Each microphone tool's ask, which the editor names as the island does. */
const SOUND_TOOL_PATCH: Record<SoundTool, (isOn: boolean) => ToolPanelPatch> = {
  autoVolume: (autoVolume) => ({ autoVolume }),
  reduceNoise: (reduceNoise) => ({ reduceNoise }),
  studioDucking: (studioDucking) => ({ studioDucking }),
  studioSound: (studioSound) => ({ studioSound }),
  vocalCleanup: (vocalCleanup) => ({ vocalCleanup }),
};

export function soundIslandContent({
  change,
  snapshot,
}: Inputs): EditorIslandContent | null {
  const tracks = snapshot.soundTracks ?? [];
  if (tracks.length === 0) return null;
  return soundIsland({
    onDownloadStudioDucking: () => {
      change({ downloadStudioDucking: true });
    },
    onDownloadStudioSound: () => {
      change({ downloadStudioSound: true });
    },
    onToolChange: (tool, isOn) => {
      change(SOUND_TOOL_PATCH[tool](isOn));
    },
    onVolumeChange: (trackId, decibels) => {
      change({ trackVolume: { decibels, trackId } });
    },
    tracks,
  });
}

export function tracksIslandContent({
  change,
  snapshot,
}: Inputs): EditorIslandContent | null {
  const tracks = snapshot.tracks ?? [];
  if (tracks.length === 0) return null;
  return tracksIsland({
    onEnabledChange: (trackId, isEnabled) => {
      change({ trackEnabled: { isEnabled, trackId } });
    },
    tracks,
  });
}
