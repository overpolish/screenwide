// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  presetDrawing,
  type ScenePreset,
} from "../../glyphs/scene-layout-drawing";
import { presetName } from "../../island/scene/scene-names";
import { type Span } from "../timeline-scale";

import { type TimelineScene } from "./scene-clip";

// Scenes behind the scene stories: a 16:10 screen and a round 16:9 camera
// on a 16:9 video.

const SHAPE = {
  aspect: 16 / 9,
  cameraAspect: 16 / 9,
  cameraRadius: 50,
  screen: { height: 0.88, width: 0.792, x: 0.104, y: 0.06 },
  screenAspect: 16 / 10,
};

export const storyScene = (
  id: string,
  preset: ScenePreset,
  span: Span,
): TimelineScene => ({
  ...span,
  drawing: presetDrawing(preset, {}, SHAPE),
  id,
  name: presetName(preset),
});
