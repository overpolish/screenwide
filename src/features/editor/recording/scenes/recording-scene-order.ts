// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Arrangement } from "../../annotations/annotation-order";

import { RecordingSceneClip } from "./recording-scenes";

export type ScenePane = "camera" | "screen";

/** `clip` with its camera drawn behind the screen, or in front. */
const withCameraBehind = (
  clip: RecordingSceneClip,
  cameraBehind: boolean,
): RecordingSceneClip => {
  if (!clip.boxes) return clip;
  const { cameraBehind: _cameraBehind, ...boxes } = clip.boxes;
  return {
    ...clip,
    boxes: cameraBehind ? { ...boxes, cameraBehind: true } : boxes,
  };
};

/** Whether `pane` is drawn in front of the other in `clip`. */
const isInFront = (clip: RecordingSceneClip, pane: ScenePane) =>
  (pane === "camera") !== Boolean(clip.boxes?.cameraBehind);

/**
 * The moves a menu on `pane` offers through `clip`'s order. Only a custom
 * scene with a box for each pane orders them, and with two panes the step
 * and the jump go the same way, so each is offered with the other.
 */
export const scenePaneArrangements = (
  clip: RecordingSceneClip,
  pane: ScenePane,
) => {
  const orderable = Boolean(clip.boxes?.camera);
  const inFront = isInFront(clip, pane);
  return {
    canBringForward: orderable && !inFront,
    canSendBackward: orderable && inFront,
  };
};

/** `clip` with `pane` moved by `move` through the order of its two panes;
 * a scene that does not order them is left as it is. */
export const arrangedScenePane = (
  clip: RecordingSceneClip,
  pane: ScenePane,
  move: Arrangement,
): RecordingSceneClip => {
  if (!clip.boxes?.camera) return clip;
  const toFront = move === "front" || move === "forward";
  return withCameraBehind(clip, pane === "camera" ? !toFront : toFront);
};

/**
 * `clip` with its two panes traded: each takes the other's box and the other's
 * place in the order, and keeps its own framing and radius. Swapping again
 * gives the scene back. A scene without a box for each pane is left as it is.
 */
export const swappedScenePanes = (
  clip: RecordingSceneClip,
): RecordingSceneClip => {
  const boxes = clip.boxes;
  if (!boxes?.camera) return clip;
  return withCameraBehind(
    { ...clip, boxes: { camera: boxes.screen, screen: boxes.camera } },
    !boxes.cameraBehind,
  );
};
