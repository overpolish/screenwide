// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SceneVariant } from "../recording/scenes/recording-scene-variant";
import {
  RecordingScenePreset,
  SceneFraming,
} from "../recording/scenes/recording-scenes";

import { SchematicComposition } from "./scene/scene-schematic";

/**
 * What the Scene panel shows: the scene under the playhead, which is the one
 * a press changes, or none, where a press adds one there.
 */
export type ToolPanelScene = {
  /** Where the scene under the playhead puts its panes, as shares of the
   * canvas, where it is custom; null for a preset's boxes or no scene. */
  boxes: SchematicComposition | null;
  /** The camera's width over its height, which the pairs keep it at. */
  cameraAspect: number;
  /** The finished frame's width over its height, which the drawings of each
   * scene are made at. */
  canvasAspect: number;
  /** The recording's own composition as shares of the canvas, which a full
   * scene keeps. */
  composition: SchematicComposition;
  /** What part of the selected pane the scene under the playhead shows, null
   * where there is no scene. */
  framing: SceneFraming | null;
  /** The pane `framing` belongs to: the camera while it is selected and the
   * scene places it, else the screen. */
  framingPane: "camera" | "screen";
  /** Whether the recording has a camera for an arrangement to place. */
  hasCamera: boolean;
  hasSceneAtPlayhead: boolean;
  /** Whether the camera is drawn into the screen's video, without which the
   * scenes that place it stand idle. */
  isBaked: boolean;
  preset: RecordingScenePreset | null;
  /** The selected pane's corner radius in the scene under the playhead, in
   * percent of its shorter side, null where there is no scene. */
  radius: number | null;
  /** The screen's visible width over its height. */
  screenAspect: number;
  /** The options of the scene under the playhead's preset, null where there
   * is no scene. */
  variant: SceneVariant | null;
};
