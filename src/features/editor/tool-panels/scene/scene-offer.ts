// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RECORDING_SCENE_PRESETS } from "../../recording/scenes/recording-scene-labels";
import { SceneTemplate } from "../../recording/scenes/recording-scene-template";
import { ArrangedScenePreset } from "../../recording/scenes/recording-scenes";
import { ToolPanelScene } from "../tool-panel-scene";
import { ToolPanelSnapshot } from "../tool-panel-store";

/** What a scene can be laid out as, beside Custom, which is always offered. */
export type SceneOffer = {
  /** Whether the camera is drawn into the screen's video, which a scene
   * needs to place it. */
  canPlaceCamera: boolean;
  /** Whether the output is one video. It is read from the panel's own
   * answer, which flips at once; the scene's own flag follows only once the
   * preview has rebuilt. */
  isCombined: boolean;
  presets: ArrangedScenePreset[];
  templates: SceneTemplate[];
};

/** The presets and templates `scene` can be laid out as: those that place a
 * camera only while one is drawn into the video. */
export const sceneOffer = (
  scene: ToolPanelScene,
  {
    cameraOutput,
    sceneTemplates,
  }: Pick<ToolPanelSnapshot, "cameraOutput" | "sceneTemplates">,
): SceneOffer => {
  const isCombined = cameraOutput ? cameraOutput === "combined" : scene.isBaked;
  const canPlaceCamera = scene.hasCamera && isCombined;
  return {
    canPlaceCamera,
    isCombined,
    presets: canPlaceCamera ? RECORDING_SCENE_PRESETS : [],
    templates: sceneTemplates.filter(
      (template) => canPlaceCamera || !template.boxes.camera,
    ),
  };
};
