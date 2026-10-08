// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import { ArrangedScenePreset, RecordingSceneClip } from "./recording-scenes";

/** What the lane and the Scene panel call a scene made custom. */
export const sceneCustomLabel = () => t("editor-scene-custom");

/** What the lane and the Scene panel call a preset. */
export const scenePresetLabel = (preset: ArrangedScenePreset) => {
  switch (preset) {
    case "camera-only":
      return t("editor-scene-camera-only");
    case "picture-in-picture":
      return t("editor-scene-picture-in-picture");
    case "screen-only":
      return t("editor-scene-screen-only");
    case "split-two-thirds":
      return t("editor-scene-side-by-side");
    case "stacked":
      return t("editor-scene-stacked");
  }
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
  if (clip.preset === "full" && (clip.screen?.zoom ?? 1) > 1)
    return t("editor-scene-zoom");
  return clip.boxes || clip.preset === "full"
    ? sceneCustomLabel()
    : scenePresetLabel(clip.preset);
};
