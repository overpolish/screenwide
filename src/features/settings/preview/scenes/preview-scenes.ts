// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ANNOTATE_SCENE } from "./annotate-scene";
import { OCR_SCENE } from "./ocr-scene";
import {
  AUTO_ZOOM_SCENE,
  DESKTOP_ICONS_SCENE,
  FRAME_RATE_SCENE,
  INCLUDE_SCREENWIDE_SCENE,
} from "./recording-scenes";
import { RULER_SCENE } from "./ruler-scene";
import { type RecordingScene, TOOL_PICTURE, type ToolScene } from "./scene-kit";

import type { PreviewScene } from "../../../../bindings/PreviewScene";

// Every scene a setting can be shown in, by the app's name for it, and the
// values that setting gives it. The pictures and the pointer are the
// scene's own.

/** A scene and its setting's values: the app's `PreviewScene` without the
 * scripted pointer, which comes with the scene. */
export type SettingsPreviewScene = PreviewScene extends infer Scene
  ? Scene extends unknown
    ? Omit<Scene, "script">
    : never
  : never;

const SCENES: Record<PreviewScene["kind"], ToolScene | RecordingScene> = {
  annotate: ANNOTATE_SCENE,
  autoZoom: AUTO_ZOOM_SCENE,
  desktopIcons: DESKTOP_ICONS_SCENE,
  frameRate: FRAME_RATE_SCENE,
  includeScreenwide: INCLUDE_SCREENWIDE_SCENE,
  ruler: RULER_SCENE,
  textRecognition: OCR_SCENE,
};

export const sceneOf = (scene: SettingsPreviewScene) => SCENES[scene.kind];

/** The size the scene's pictures are painted at, in points. */
export function pictureSize(scene: SettingsPreviewScene) {
  const definition = sceneOf(scene);
  return definition.kind === "tool" ? TOOL_PICTURE : definition.size;
}

/** The scene as the app takes it, with its scripted pointer. */
export function nativeScene(scene: SettingsPreviewScene): PreviewScene {
  const definition = sceneOf(scene);
  const script =
    definition.kind === "recording" ? definition.script : undefined;
  if (scene.kind === "autoZoom" || scene.kind === "frameRate") {
    if (!script) throw new Error(`The ${scene.kind} scene has no pointer`);
    return { ...scene, script };
  }
  return scene;
}

/** Whether the picture shown outside the app, the scene's last state, has
 * the scene's layer on it: Screenwide's windows, or the desktop's icons. */
export const showsLayer = (scene: SettingsPreviewScene) =>
  scene.kind === "includeScreenwide"
    ? scene.included
    : scene.kind === "desktopIcons" && !scene.hidden;
