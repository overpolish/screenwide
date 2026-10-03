// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  fittedTemplateBoxes,
  SceneTemplate,
} from "../../recording/scenes/recording-scene-template";

import { ScenePresetTile } from "./scene-preset-tile";
import { useSceneTemplateMenu } from "./use-scene-template-menu";

/**
 * The scene templates on offer after the presets, each drawn where it puts
 * the panes on this recording's canvas. A press lays the scene out so; a
 * right click offers to remove the template.
 */
export function SceneTemplateTiles({
  canvasAspect,
  isDisabled,
  onChoose,
  onRemove,
  selectedId,
  templates,
}: {
  canvasAspect: number;
  isDisabled: boolean;
  onChoose: (template: SceneTemplate) => void;
  onRemove: (templateId: string) => void;
  selectedId: string | null;
  templates: readonly SceneTemplate[];
}) {
  const showMenu = useSceneTemplateMenu(onRemove);
  return templates.map((template) => {
    const boxes = fittedTemplateBoxes(template, canvasAspect);
    return (
      <ScenePresetTile
        canvasAspect={canvasAspect}
        id={template.id}
        isDisabled={isDisabled}
        isSelected={selectedId === template.id}
        key={template.id}
        label={template.name}
        onContextMenu={(event) => {
          event.preventDefault();
          if (isDisabled) return;
          void showMenu(
            template.id,
            event.currentTarget.getBoundingClientRect(),
          );
        }}
        onPress={() => {
          onChoose(template);
        }}
        schematic={{
          camera: boxes.camera ?? null,
          cameraBehind: boxes.cameraBehind ?? false,
          screen: boxes.screen,
        }}
      />
    );
  });
}
