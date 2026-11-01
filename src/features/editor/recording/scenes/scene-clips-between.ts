// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { between, rotationBetween } from "../../preview/picture-between";

import { ScenePane } from "./recording-scene-order";
import { RecordingSceneClip } from "./recording-scenes";

import type { SceneBox } from "../../../../bindings/SceneBox";
import type { SceneFraming } from "../../../../bindings/SceneFraming";

// A scene's padding, corners, turn and framing chosen on an island are drawn
// as the picture moving to them, as the recording's own are
// (`picture-tween.ts`). What lays a scene out, its preset and its variant,
// is drawn as `to` has it: the preview arranges a preset's boxes itself.

const boxBetween = (from: SceneBox, to: SceneBox, share: number): SceneBox => ({
  height: between(from.height, to.height, share),
  width: between(from.width, to.width, share),
  x: between(from.x, to.x, share),
  y: between(from.y, to.y, share),
});

/** A framing between `from` and `to`; one left unset is drawn as `to` has
 * it, as an unset framing is the preset's own. */
const framingBetween = (
  from: SceneFraming | undefined,
  to: SceneFraming | undefined,
  share: number,
) =>
  from && to
    ? {
        focusX: between(from.focusX, to.focusX, share),
        focusY: between(from.focusY, to.focusY, share),
        zoom: between(from.zoom, to.zoom, share),
      }
    : to;

/** How round a scene draws a pane: the recording's own radius where the
 * scene sets none. */
type RadiusOf = (clip: RecordingSceneClip, pane: ScenePane) => number;

/** `to`'s scene drawn `share` of the way from `from`'s. */
const clipBetween = (
  { from, to }: { from: RecordingSceneClip; to: RecordingSceneClip },
  share: number,
  radiusOf: RadiusOf,
): RecordingSceneClip => {
  const radius = (pane: ScenePane) =>
    from.radius?.[pane] === undefined && to.radius?.[pane] === undefined
      ? undefined
      : between(radiusOf(from, pane), radiusOf(to, pane), share);
  const [screenRadius, cameraRadius] = [radius("screen"), radius("camera")];
  const rotation = rotationBetween(from.rotation, to.rotation, share);
  const inset =
    from.inset === undefined && to.inset === undefined
      ? undefined
      : between(from.inset ?? 0, to.inset ?? 0, share);
  const camera = framingBetween(from.camera, to.camera, share);
  const screen = framingBetween(from.screen, to.screen, share);
  return {
    ...to,
    ...(from.boxes && to.boxes
      ? {
          boxes: {
            ...to.boxes,
            ...(from.boxes.camera && to.boxes.camera
              ? {
                  camera: boxBetween(from.boxes.camera, to.boxes.camera, share),
                }
              : {}),
            screen: boxBetween(from.boxes.screen, to.boxes.screen, share),
          },
        }
      : {}),
    ...(camera ? { camera } : {}),
    ...(inset === undefined ? {} : { inset }),
    ...(screenRadius === undefined && cameraRadius === undefined
      ? {}
      : {
          radius: {
            ...(cameraRadius === undefined ? {} : { camera: cameraRadius }),
            ...(screenRadius === undefined ? {} : { screen: screenRadius }),
          },
        }),
    ...(rotation ? { rotation } : {}),
    ...(screen ? { screen } : {}),
  };
};

/** Every scene of `to` drawn part of the way from how `from` has it; a scene
 * only `to` has is drawn as it is. Corners a scene starts or stops rounding
 * move from or to the recording's own, which `radiusOf` falls back on. */
export const sceneClipsBetween =
  (radiusOf: RadiusOf) =>
  (from: RecordingSceneClip[], to: RecordingSceneClip[], share: number) =>
    to.map((clip) => {
      const was = from.find((candidate) => candidate.id === clip.id);
      return was ? clipBetween({ from: was, to: clip }, share, radiusOf) : clip;
    });
