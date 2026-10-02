// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { recordingSceneControls } from "../../recording/scenes/recording-scene-channel";
import {
  nextSceneTemplateName,
  SceneTemplate,
} from "../../recording/scenes/recording-scene-template";
import { sceneNeedsCamera } from "../../recording/scenes/recording-scenes";
import { EditorKind } from "../../types";
import { ToolPanelHandlers } from "../tool-panel-handlers";

/**
 * What the Scene panel's asks do in `workspace`: they reach the preview,
 * which knows the scene under its playhead. An arrangement only places the
 * camera in a baked composition, so with baking off those scenes stand idle;
 * choosing one, new or existing, bakes the camera again in the same edit.
 * The templates are a preference rather than part of the recording:
 * `sceneTemplates` are those kept now and `saveTemplates` keeps a new list.
 */
export const scenePanelHandlers = (
  workspace: EditorKind,
  {
    bakeCamera,
    onBakeCameraChange,
    saveTemplates,
    sceneTemplates,
  }: {
    bakeCamera: boolean;
    saveTemplates: (templates: SceneTemplate[]) => void;
    sceneTemplates: SceneTemplate[];
    onBakeCameraChange?: (bake: boolean) => void;
  },
): Pick<
  ToolPanelHandlers,
  | "onSceneCustomize"
  | "onSceneDelete"
  | "onSceneFramingChange"
  | "onSceneFramingReset"
  | "onScenePresetChoose"
  | "onSceneRadiusChange"
  | "onSceneTemplateChoose"
  | "onSceneTemplateRemove"
  | "onSceneTemplateSave"
  | "onSceneVariantChange"
> => ({
  onSceneCustomize: () => {
    recordingSceneControls(workspace)?.customize();
  },
  onSceneDelete: () => {
    recordingSceneControls(workspace)?.remove();
  },
  onSceneFramingChange: (framing) => {
    recordingSceneControls(workspace)?.setFraming(framing);
  },
  onSceneFramingReset: () => {
    recordingSceneControls(workspace)?.resetFraming();
  },
  onScenePresetChoose: (preset) => {
    if (!bakeCamera && sceneNeedsCamera({ preset })) onBakeCameraChange?.(true);
    recordingSceneControls(workspace)?.choosePreset(preset);
  },
  onSceneRadiusChange: (radius) => {
    recordingSceneControls(workspace)?.setRadius(radius);
  },
  onSceneTemplateChoose: (template) => {
    if (!bakeCamera && template.boxes.camera) onBakeCameraChange?.(true);
    recordingSceneControls(workspace)?.chooseTemplate(template);
  },
  onSceneTemplateRemove: (id) => {
    saveTemplates(sceneTemplates.filter((template) => template.id !== id));
  },
  onSceneTemplateSave: () => {
    const layout = recordingSceneControls(workspace)?.templateLayout();
    if (!layout) return;
    saveTemplates([
      ...sceneTemplates,
      {
        ...layout,
        id: `template-${Date.now().toString(36)}`,
        name: nextSceneTemplateName(sceneTemplates),
      },
    ]);
  },
  onSceneVariantChange: (variant) => {
    recordingSceneControls(workspace)?.setVariant(variant);
  },
});
