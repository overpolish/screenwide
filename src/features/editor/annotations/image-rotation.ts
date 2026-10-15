// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MAX_TURN, SceneRotation } from "../layer-rotation";

import type { AnnotationShape } from "./annotations";

type ImageShape = Extract<AnnotationShape, { kind: "image" }>;

/** How an image is turned, as the Rotation row's dials show it: its tilt in
 * depth about X and Y, and its flat turn about Z, which runs
 * counter-clockwise as the image's own angle runs clockwise. */
export const imageRotation = (shape: ImageShape): SceneRotation => ({
  x: shape.tilt?.x ?? 0,
  y: shape.tilt?.y ?? 0,
  z: -(shape.angle * 180) / Math.PI,
});

/** `shape` turned to `rotation`, each angle held to what a turn can reach,
 * flat again where it is tilted neither way; `shape` itself where that
 * changes nothing. */
export const withImageRotation = (
  shape: ImageShape,
  rotation: SceneRotation,
): ImageShape => {
  if (![rotation.x, rotation.y, rotation.z].every(Number.isFinite))
    return shape;
  const held = (angle: number) =>
    Math.min(MAX_TURN, Math.max(-MAX_TURN, angle));
  const [x, y] = [held(rotation.x), held(rotation.y)];
  const angle = -(held(rotation.z) * Math.PI) / 180;
  const current = imageRotation(shape);
  if (
    x === current.x &&
    y === current.y &&
    held(rotation.z) === held(current.z)
  )
    return shape;
  const { tilt: _tilt, ...flat } = shape;
  return x === 0 && y === 0
    ? { ...flat, angle }
    : { ...flat, angle, tilt: { x, y } };
};
