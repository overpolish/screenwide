// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DEFAULT_BUBBLE_SIZE,
  DEFAULT_CAMERA_SIZE,
  DEFAULT_CORNER,
  SceneVariant,
} from "./recording-scene-variant";
import { RecordingScenePreset } from "./recording-scenes";

export type SceneRect = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** The presets that lay the panes out themselves, rather than keeping the
 * recording's own place for the screen. */
export type LaidOutScenePreset = Exclude<
  RecordingScenePreset,
  "full" | "screen-only"
>;

/** How far a picture in picture's camera reaches past the screen's corner
 * each way, as a share of its side. */
const BUBBLE_OVERHANG = 0.4;

/** The camera's side in a picture in picture, as a share of the screen's
 * height. */
const BUBBLE_SHARE = { large: 0.65, small: 0.4 } as const;

/** The camera's length in a pair over the screen's. */
const CAMERA_RATIO = { third: 0.5, "two-thirds": 2 } as const;

/**
 * The screen's and the camera's panes for `preset` on a `canvas`, for a
 * screen of `screenAspect` and a camera of `cameraAspect`, each width over
 * height, as `variant` has it, in the canvas's own units; null for a pane
 * the preset hides. The twin of `preset_panes` in
 * `src-tauri/src/editor/scenes/geometry.rs`, which places the frames the
 * preview and the export draw.
 *
 * A margin of a ninth of the canvas's shorter side is kept clear. Side by
 * side and stacked keep each pane at its own shape, half a margin apart, the
 * camera half the screen's length, a third of the pair, or twice it, second
 * unless swapped; each pane is centred on the other across the pair, so the
 * camera is shown whole rather than cropped to match the screen's edge.
 * Picture in picture is the screen at its own aspect with the camera
 * reaching past the corner it sits over by the same share each way. Each
 * pair is scaled to the largest size the margin leaves room for and centred.
 * Camera only fills the canvas.
 */
export function recordingScenePanes(
  preset: LaidOutScenePreset,
  canvas: { height: number; width: number },
  {
    cameraAspect,
    screenAspect,
    variant = {},
  }: { cameraAspect: number; screenAspect: number; variant?: SceneVariant },
): { camera: SceneRect | null; screen: SceneRect | null } {
  if (preset === "camera-only")
    return {
      camera: { height: canvas.height, width: canvas.width, x: 0, y: 0 },
      screen: null,
    };
  const margin = Math.min(canvas.width, canvas.height) / 9;
  const roomWidth = canvas.width - margin * 2;
  const roomHeight = canvas.height - margin * 2;
  const gap = margin / 2;
  const ratio = CAMERA_RATIO[variant.cameraSize ?? DEFAULT_CAMERA_SIZE];
  const swap = variant.swap ?? false;
  if (preset === "split-two-thirds") {
    // The screen's width, held by the pair's width and by the taller pane.
    const width = Math.min(
      (roomWidth - gap) / (1 + ratio),
      roomHeight / Math.max(1 / screenAspect, ratio / cameraAspect),
    );
    const height = width / screenAspect;
    const cameraWidth = width * ratio;
    const cameraHeight = cameraWidth / cameraAspect;
    const x = (canvas.width - (width + cameraWidth + gap)) / 2;
    return {
      camera: {
        height: cameraHeight,
        width: cameraWidth,
        x: swap ? x : x + width + gap,
        y: (canvas.height - cameraHeight) / 2,
      },
      screen: {
        height,
        width,
        x: swap ? x + cameraWidth + gap : x,
        y: (canvas.height - height) / 2,
      },
    };
  }
  if (preset === "stacked") {
    // The screen's height, held by the pair's height and by the wider pane.
    const height = Math.min(
      (roomHeight - gap) / (1 + ratio),
      roomWidth / Math.max(screenAspect, ratio * cameraAspect),
    );
    const width = height * screenAspect;
    const cameraHeight = height * ratio;
    const cameraWidth = cameraHeight * cameraAspect;
    const y = (canvas.height - (height + cameraHeight + gap)) / 2;
    return {
      camera: {
        height: cameraHeight,
        width: cameraWidth,
        x: (canvas.width - cameraWidth) / 2,
        y: swap ? y : y + height + gap,
      },
      screen: {
        height,
        width,
        x: (canvas.width - width) / 2,
        y: swap ? y + cameraHeight + gap : y,
      },
    };
  }
  const share = BUBBLE_SHARE[variant.size ?? DEFAULT_BUBBLE_SIZE];
  const reach = share * BUBBLE_OVERHANG;
  const height = Math.min(
    roomWidth / (screenAspect + reach),
    roomHeight / (1 + reach),
  );
  const width = height * screenAspect;
  const side = height * share;
  const overhang = side * BUBBLE_OVERHANG;
  const x = (canvas.width - (width + overhang)) / 2;
  const y = (canvas.height - (height + overhang)) / 2;
  const corner = variant.corner ?? DEFAULT_CORNER;
  const right = corner.endsWith("right");
  const bottom = corner.startsWith("bottom");
  const screenX = right ? x : x + overhang;
  const screenY = bottom ? y : y + overhang;
  return {
    camera: {
      height: side,
      width: side,
      x: right ? screenX + width - (side - overhang) : x,
      y: bottom ? screenY + height - (side - overhang) : y,
    },
    screen: { height, width, x: screenX, y: screenY },
  };
}
