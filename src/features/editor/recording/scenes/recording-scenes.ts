// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withRotation } from "../../layer-rotation";
import { insertTimedSpanClip } from "../../timeline/tracks/timed-span-clips";

import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { RecordingScenePreset } from "../../../../bindings/RecordingScenePreset";
import type { SceneBox } from "../../../../bindings/SceneBox";
import type { SceneBoxes } from "../../../../bindings/SceneBoxes";
import type { SceneFraming } from "../../../../bindings/SceneFraming";
import type { SceneRotation } from "../../../../bindings/SceneRotation";

export type {
  RecordingSceneClip,
  RecordingScenePreset,
  SceneBox,
  SceneBoxes,
  SceneFraming,
  SceneRotation,
};

/** The presets the Scene panel offers beside Custom. */
export type ArrangedScenePreset = Exclude<RecordingScenePreset, "full">;

/** The whole picture, centred: what a scene shows until it is reframed. */
export const WHOLE_FRAMING: SceneFraming = {
  focusX: 0.5,
  focusY: 0.5,
  zoom: 1,
};

/** The furthest a scene zooms into a picture. */
export const MAX_SCENE_ZOOM = 8;

/** `clip` turning the screen by `rotation`, each angle held to what a scene
 * can turn it; one that leaves it upright keeps no turn at all. */
export const rotatedSceneClip = (
  clip: RecordingSceneClip,
  rotation: SceneRotation,
): RecordingSceneClip => withRotation(clip, rotation);

/** Whether a scene places a camera, which it can only do where the camera is
 * drawn into the picture. A custom scene places one where it has a box for
 * it. */
export const sceneNeedsCamera = ({
  boxes,
  preset,
}: {
  preset: RecordingScenePreset;
  boxes?: SceneBoxes;
}) =>
  boxes
    ? boxes.camera !== undefined
    : preset !== "full" && preset !== "screen-only";

/** How long a scene added at the playhead lasts, where there is room. */
const RECORDING_SCENE_DEFAULT_MS = 5000;
/** Shorter than this, a scene would spend its whole life arriving. */
export const RECORDING_SCENE_MINIMUM_MS = 500;

/**
 * A new scene of `preset` at `atMs`, as long as the default where the free
 * stretch around the playhead allows. Null where the playhead is inside a
 * clip or the stretch is too short to hold one.
 */
export const insertRecordingSceneClip = ({
  atMs,
  clips,
  id,
  lengthMs,
  preset,
}: {
  atMs: number;
  clips: readonly RecordingSceneClip[];
  id: string;
  lengthMs: number;
  preset: RecordingScenePreset;
}) =>
  insertTimedSpanClip({
    atMs,
    clips,
    defaultMs: RECORDING_SCENE_DEFAULT_MS,
    lengthMs,
    make: (span): RecordingSceneClip => ({ ...span, id, preset }),
    minimumMs: RECORDING_SCENE_MINIMUM_MS,
  });
