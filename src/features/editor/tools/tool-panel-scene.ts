// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SceneVariant } from "../recording/scenes/recording-scene-variant";
import {
  RecordingScenePreset,
  SceneFraming,
  SceneRotation,
} from "../recording/scenes/recording-scenes";

import { SchematicComposition } from "./scene/scene-schematic";

import type { Background } from "./frame/background/background";

/** A pane of the scene under the playhead: what part of it the scene
 * shows, how round its corners are, in percent of its shorter side, and how
 * far it is padded, as a share of its box's shorter side (the screen's
 * alone; the camera is never padded). */
export type ToolPanelScenePane = {
  framing: SceneFraming;
  inset: number;
  /** Whether the pane sits in the box the scene's layout gives it, so
   * Reset placement has nothing to put back: always so for a preset's. */
  isAtLayout: boolean;
  radius: number;
};

/**
 * What the Scene panel shows: the scene under the playhead, which is the one
 * a press changes, or none, where a press adds one there.
 */
export type ToolPanelScene = {
  /** What the canvas sits on in the scene under the playhead, over the
   * recording's own; null where it shows the recording's or there is no
   * scene. */
  background: Background | null;
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
  /** Whether the scene under the playhead shows the cursor, over the
   * recording's own choice; null where it makes none or there is no
   * scene. */
  cursor: boolean | null;
  /** Whether the scene under the playhead has the camera follow the face. */
  followsFace: boolean;
  /** What part of the selected pane the scene under the playhead shows, null
   * where there is no scene. */
  framing: SceneFraming | null;
  /** The pane `framing` belongs to: the camera while it is selected and the
   * scene places it, else the screen. */
  framingPane: "camera" | "screen";
  /** Whether the timeline holds any auto zoom for Clear all to take off. */
  hasAutoZooms: boolean;
  /** Whether the recording has a camera for an arrangement to place. */
  hasCamera: boolean;
  hasSceneAtPlayhead: boolean;
  /** Whether the timeline holds any scene, for which the Frame island keeps
   * room for a scene's note and action as the playhead moves. */
  hasScenes: boolean;
  /** Whether the camera is drawn into the screen's video, without which the
   * scenes that place it stand idle. */
  isBaked: boolean;
  /** Each pane the scene under the playhead shows, null where it hides that
   * pane or there is no scene: the Scene island frames either. */
  panes: {
    camera: ToolPanelScenePane | null;
    screen: ToolPanelScenePane | null;
  };
  preset: RecordingScenePreset | null;
  /** The selected pane's corner radius in the scene under the playhead, in
   * percent of its shorter side, null where there is no scene. */
  radius: number | null;
  /** How the scene under the playhead turns the screen, in degrees about
   * each axis, whichever pane is selected; null where there is no scene. */
  rotation: SceneRotation | null;
  /** The scene under the playhead's own id, null where there is none. */
  sceneId: string | null;
  /** The screen's visible width over its height. */
  screenAspect: number;
  /** The options of the scene under the playhead's preset, null where there
   * is no scene. */
  variant: SceneVariant | null;
};
