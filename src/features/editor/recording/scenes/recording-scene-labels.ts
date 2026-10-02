// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrangedScenePreset, RecordingSceneClip } from "./recording-scenes";

/** What the lane and the Scene panel call a scene made custom. */
export const RECORDING_SCENE_CUSTOM_LABEL = "Custom";

export const RECORDING_SCENE_LABELS: Record<ArrangedScenePreset, string> = {
  "camera-only": "Camera only",
  "picture-in-picture": "Picture in picture",
  "screen-only": "Screen only",
  "split-two-thirds": "Side by side",
  stacked: "Stacked",
};

/** The presets on offer after Custom, in the order the Scene panel shows
 * them. */
export const RECORDING_SCENE_PRESETS: ArrangedScenePreset[] = [
  "split-two-thirds",
  "stacked",
  "picture-in-picture",
  "camera-only",
  "screen-only",
];

/** What the lane calls `clip`. A scene kept in the recording's own boxes
 * that zooms into the screen reads as the zoom it is. */
export const recordingSceneLabel = (clip: RecordingSceneClip) => {
  if (clip.preset === "full" && (clip.screen?.zoom ?? 1) > 1) return "Zoom";
  return clip.boxes || clip.preset === "full"
    ? RECORDING_SCENE_CUSTOM_LABEL
    : RECORDING_SCENE_LABELS[clip.preset];
};
