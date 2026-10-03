// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { recordingSceneControls } from "../../recording/scenes/recording-scene-channel";
import {
  nextSceneTemplateName,
  SceneTemplate,
} from "../../recording/scenes/recording-scene-template";
import { EditorKind } from "../../types";
import { ToolPanelHandlers } from "../tool-panel-handlers";

/**
 * What the Scene panel's asks do in `workspace`: they reach the preview,
 * which knows the scene under its playhead. An arrangement only places the
 * camera when it is drawn into the screen's video, so with separate files
 * those scenes stand idle; choosing a scene never changes the output.
 * The templates are a preference rather than part of the recording:
 * `sceneTemplates` are those kept now and `saveTemplates` keeps a new list.
 */
export const scenePanelHandlers = (
  workspace: EditorKind,
  {
    saveTemplates,
    sceneTemplates,
  }: {
    saveTemplates: (templates: SceneTemplate[]) => void;
    sceneTemplates: SceneTemplate[];
  },
): Pick<
  ToolPanelHandlers,
  | "onSceneAutoZoom"
  | "onSceneCustomize"
  | "onSceneDelete"
  | "onSceneFramingChange"
  | "onSceneFramingReset"
  | "onScenePanesSwap"
  | "onScenePresetChoose"
  | "onSceneRadiusChange"
  | "onSceneTemplateChoose"
  | "onSceneTemplateRemove"
  | "onSceneTemplateSave"
  | "onSceneVariantChange"
> => ({
  onSceneAutoZoom: () => {
    recordingSceneControls(workspace)?.autoZoom();
  },
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
  onScenePanesSwap: () => {
    recordingSceneControls(workspace)?.swapPanes();
  },
  onScenePresetChoose: (preset) => {
    recordingSceneControls(workspace)?.choosePreset(preset);
  },
  onSceneRadiusChange: (radius) => {
    recordingSceneControls(workspace)?.setRadius(radius);
  },
  onSceneTemplateChoose: (template) => {
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
