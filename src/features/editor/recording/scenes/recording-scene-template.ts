// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingSceneClip,
  SceneBox,
  SceneBoxes,
  SceneFraming,
  SceneRadius,
} from "./recording-scenes";

/**
 * A custom scene's layout kept under a name for any recording: its boxes as
 * shares of the canvas it was made on, that canvas's shape, and how far it
 * zooms into and rounds each pane. Where the part of a picture a zoom shows
 * is about one recording, so a template keeps the zoom and not the position.
 * The twin of `SceneTemplate` in `src-tauri/src/editor/scenes/template.rs`,
 * which keeps it sound in the settings.
 */
export type SceneTemplate = {
  boxes: SceneBoxes;
  /** The width over the height of the canvas the layout was made on. */
  canvasAspect: number;
  id: string;
  name: string;
  radius?: SceneRadius;
  zoom?: { camera?: number; screen?: number };
};

/** What a template keeps of a scene, before it has a name. */
export type SceneTemplateLayout = Omit<SceneTemplate, "id" | "name">;

/** The layout `clip` would be kept as on a canvas of `canvasAspect`, or null
 * for a scene that is not custom, which has no boxes of its own to keep. */
export const sceneTemplateLayout = (
  clip: RecordingSceneClip,
  canvasAspect: number,
): SceneTemplateLayout | null => {
  if (!clip.boxes) return null;
  const zoom = {
    ...(clip.screen && clip.screen.zoom > 1
      ? { screen: clip.screen.zoom }
      : {}),
    ...(clip.camera && clip.boxes.camera && clip.camera.zoom > 1
      ? { camera: clip.camera.zoom }
      : {}),
  };
  return {
    boxes: clip.boxes,
    canvasAspect,
    ...(clip.radius ? { radius: clip.radius } : {}),
    ...(Object.keys(zoom).length > 0 ? { zoom } : {}),
  };
};

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

const centred = (zoom: number): SceneFraming => ({
  focusX: 0.5,
  focusY: 0.5,
  zoom,
});

/**
 * `clip` laid out as `template` on a canvas of `canvasAspect`: a custom
 * scene in the template's boxes, rounded and zoomed as it was, each zoom
 * from the middle of its picture. Nothing ties the scene to the template
 * afterwards.
 */
export const clipFromTemplate = (
  clip: RecordingSceneClip,
  { canvasAspect, template }: { canvasAspect: number; template: SceneTemplate },
): RecordingSceneClip => {
  const {
    boxes: _boxes,
    camera: _camera,
    radius: _radius,
    screen: _screen,
    ...rest
  } = clip;
  const boxes = fittedTemplateBoxes(template, canvasAspect);
  const { zoom } = template;
  return {
    ...rest,
    boxes,
    ...(zoom?.screen ? { screen: centred(zoom.screen) } : {}),
    ...(zoom?.camera && boxes.camera ? { camera: centred(zoom.camera) } : {}),
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
