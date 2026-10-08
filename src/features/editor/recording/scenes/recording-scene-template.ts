// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingSceneClip, SceneBox, SceneBoxes } from "./recording-scenes";

import type { SceneTemplate } from "../../../../bindings/SceneTemplate";

export type { SceneTemplate };

/** What a template keeps of a scene, before it has a name. */
export type SceneTemplateLayout = Omit<SceneTemplate, "id" | "name">;

/** The layout `clip` would be kept as on a canvas of `canvasAspect`, or null
 * for a scene that is not custom, which has no boxes of its own to keep. */
export const sceneTemplateLayout = (
  clip: RecordingSceneClip,
  canvasAspect: number,
): SceneTemplateLayout | null =>
  clip.boxes
    ? {
        boxes: clip.boxes,
        canvasAspect,
        ...(clip.radius ? { radius: clip.radius } : {}),
      }
    : null;

/** The next of the names templates are given, one past the highest
 * "Template N" kept. */
export const nextSceneTemplateName = (templates: readonly SceneTemplate[]) => {
  const numbers = templates.map(
    (template) => Number(/^Template (\d+)$/u.exec(template.name)?.[1]) || 0,
  );
  return `Template ${String(Math.max(0, ...numbers) + 1)}`;
};

/**
 * `template`'s boxes on a canvas of `canvasAspect`, as shares of it, in the
 * template's order. The canvas the layout was made on is fitted inside this
 * one, as large as fits and centred, and every box carried with it, so each
 * keeps its shape and its size beside the other: the same canvas shape gets
 * the layout as it was made, another gets it smaller with room around it.
 */
export const fittedTemplateBoxes = (
  template: SceneTemplateLayout,
  canvasAspect: number,
): SceneBoxes => {
  // In units of this canvas's height, so it is `canvasAspect` wide.
  const width = Math.min(canvasAspect, template.canvasAspect);
  const height = width / template.canvasAspect;
  const left = (canvasAspect - width) / 2;
  const top = (1 - height) / 2;
  const fitted = (box: SceneBox): SceneBox => ({
    height: box.height * height,
    width: (box.width * width) / canvasAspect,
    x: (left + box.x * width) / canvasAspect,
    y: top + box.y * height,
  });
  return {
    screen: fitted(template.boxes.screen),
    ...(template.boxes.camera ? { camera: fitted(template.boxes.camera) } : {}),
    ...(template.boxes.cameraBehind ? { cameraBehind: true } : {}),
  };
};

/**
 * `clip` laid out as `template` on a canvas of `canvasAspect`: a custom
 * scene in the template's boxes, rounded as it was, each pane still zoomed
 * into what the scene showed of it, as a preset keeps it. Nothing ties the
 * scene to the template afterwards.
 */
export const clipFromTemplate = (
  clip: RecordingSceneClip,
  { canvasAspect, template }: { canvasAspect: number; template: SceneTemplate },
): RecordingSceneClip => {
  const { radius: _radius, ...rest } = clip;
  return {
    ...rest,
    boxes: fittedTemplateBoxes(template, canvasAspect),
    ...(template.radius ? { radius: template.radius } : {}),
  };
};

const sameBox = (a: SceneBox | undefined, b: SceneBox | undefined) =>
  a === b ||
  (a !== undefined &&
    b !== undefined &&
    Math.abs(a.x - b.x) < 1e-6 &&
    Math.abs(a.y - b.y) < 1e-6 &&
    Math.abs(a.width - b.width) < 1e-6 &&
    Math.abs(a.height - b.height) < 1e-6);

/** Whether `boxes` are where, and in the order, `template` puts the panes on
 * a canvas of `canvasAspect`, which marks its tile chosen. */
export const isTemplateLayout = (
  boxes: SceneBoxes,
  { canvasAspect, template }: { canvasAspect: number; template: SceneTemplate },
) => {
  const fitted = fittedTemplateBoxes(template, canvasAspect);
  return (
    sameBox(boxes.screen, fitted.screen) &&
    sameBox(boxes.camera, fitted.camera) &&
    Boolean(boxes.cameraBehind) === Boolean(fitted.cameraBehind)
  );
};

/** Which of `templates` lays the panes out where a custom scene's `boxes`
 * are on a canvas of `canvasAspect`, by id; null for none, and for a scene
 * with no boxes of its own. */
export const chosenSceneTemplate = (
  templates: readonly SceneTemplate[],
  {
    boxes,
    canvasAspect,
  }: {
    boxes: {
      camera: SceneBox | null;
      screen: SceneBox | null;
      cameraBehind?: boolean;
    } | null;
    canvasAspect: number;
  },
) => {
  if (!boxes?.screen) return null;
  const sceneBoxes: SceneBoxes = {
    screen: boxes.screen,
    ...(boxes.camera ? { camera: boxes.camera } : {}),
    ...(boxes.cameraBehind ? { cameraBehind: true } : {}),
  };
  return (
    templates.find((template) =>
      isTemplateLayout(sceneBoxes, { canvasAspect, template }),
    )?.id ?? null
  );
};
