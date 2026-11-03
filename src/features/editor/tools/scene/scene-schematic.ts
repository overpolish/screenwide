// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SceneRect } from "../../recording/scenes/recording-scene-geometry";

/** Where a scene puts the screen and the camera as shares of the canvas,
 * null for a pane it does not show, and whether the camera is drawn behind
 * the screen rather than in front of it. A pane's `x` and `width` are shares
 * of the canvas's width, `y` and `height` of its height. */
export type SchematicComposition = {
  camera: SceneRect | null;
  screen: SceneRect | null;
  cameraBehind?: boolean;
};
