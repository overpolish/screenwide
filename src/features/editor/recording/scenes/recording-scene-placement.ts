// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings } from "../../types";

import { drawnCameraFrame, placedPicture } from "./recording-scene-framing";
import { SceneRect, recordingScenePanes } from "./recording-scene-geometry";
import {
  RecordingSceneClip,
  SceneBox,
  WHOLE_FRAMING,
  sceneNeedsCamera,
} from "./recording-scenes";

export type Picture = { width: number; x: number; y: number };

/** Where the panes are, in the canvas's output pixels: the screen's box and
 * the whole screen image behind it by its corner and width, then the
 * camera's box and the camera picture behind it by its centre and width;
 * each pane's corner radius in percent of its box's shorter side; how
 * opaque each pane is drawn, zero for a pane the scene hides; and how far the
 * camera is drawn in front of the screen, one in front and zero behind. The
 * twin of `Placement` in `src-tauri/src/editor/scenes/placement.rs`. */
export type Placement = {
  camera: Picture;
  cameraFront: number;
  frame: SceneRect;
  image: Picture;
  opacity: { camera: number; screen: number };
  radius: { camera: number; screen: number };
  screen: SceneRect;
};

export type SceneCamera = { aspect: number; overlay: CameraOverlaySettings };

type Canvas = { height: number; width: number };

/** Where the recording's own composition puts the panes, outside every
 * scene. */
export const basePlacement = (
  output: ScreenshotOutputSettings,
  camera: SceneCamera | null,
): Placement => ({
  camera: camera
    ? {
        width: camera.overlay.cameraWidth,
        x: camera.overlay.cameraX,
        y: camera.overlay.cameraY,
      }
    : { width: 0, x: 0, y: 0 },
  cameraFront: 1,
  // Where the camera is drawn rather than where its box is stored, so a
  // scene arriving starts from the camera on screen.
  frame: camera
    ? drawnCameraFrame(camera.overlay, camera.aspect)
    : { height: 0, width: 0, x: 0, y: 0 },
  image: { width: output.imageWidth, x: output.imageX, y: output.imageY },
  opacity: { camera: 1, screen: 1 },
  radius: {
    camera: camera?.overlay.radiusPercent ?? 0,
    screen: output.radiusPercent,
  },
  screen: {
    height: output.cropHeight,
    width: output.cropWidth,
    x: output.cropX,
    y: output.cropY,
  },
});

/** A custom scene's box on a canvas of output pixels. */
const onCanvas = (box: SceneBox, canvas: Canvas): SceneRect => ({
  height: box.height * canvas.height,
  width: box.width * canvas.width,
  x: box.x * canvas.width,
  y: box.y * canvas.height,
});

/**
 * The boxes `clip` puts the screen and the camera in, in output pixels, from
 * its custom boxes or else its preset, and which of the two it shows. The
 * camera's box is null where the scene places none and the recording's own
 * box stays. A pane the preset hides keeps the recording's own place,
 * unseen; a custom scene shows both.
 */
export const sceneBoxes = (
  clip: RecordingSceneClip,
  {
    base,
    camera,
    canvas,
  }: { base: Placement; camera: SceneCamera | null; canvas: Canvas },
) => {
  const { boxes } = clip;
  if (boxes)
    return {
      camera: boxes.camera ? onCanvas(boxes.camera, canvas) : null,
      screen: onCanvas(boxes.screen, canvas),
      shown: { camera: true, screen: true },
    };
  if (clip.preset === "full" || clip.preset === "screen-only")
    return {
      camera: null,
      screen: base.screen,
      shown: { camera: clip.preset === "full", screen: true },
    };
  // Without a camera no preset that places one plays, so any shape will do;
  // `arrange.rs` takes the same.
  const panes = recordingScenePanes(clip.preset, canvas, {
    cameraAspect: camera?.aspect ?? 16 / 9,
    screenAspect: base.screen.width / base.screen.height,
    variant: clip.variant,
  });
  return {
    camera: panes.camera,
    screen: panes.screen ?? base.screen,
    shown: { camera: panes.camera !== null, screen: panes.screen !== null },
  };
};

/** Where `clip` settles the panes, or null for an arrangement that needs a
 * camera where none is drawn into the picture: that scene stands idle. The
 * twin of `Base::target` in `arrange.rs`. */
export const targeting =
  ({
    base,
    camera,
    canvas,
  }: {
    base: Placement;
    camera: SceneCamera | null;
    canvas: Canvas;
  }) =>
  (clip: RecordingSceneClip): Placement | null => {
    const crop = base.screen;
    const aspect = crop.width / crop.height;
    const boxes = sceneBoxes(clip, { base, camera, canvas });
    if (boxes.camera && !camera) return null;
    const { screen, shown } = boxes;
    const frame = boxes.camera ?? base.frame;
    // The screen's framing places the visible crop as if it were the
    // picture, so a zoom shows less of what the crop shows; the whole image
    // follows it.
    const picture = placedPicture(clip.screen ?? WHOLE_FRAMING, screen, aspect);
    const scale = picture.width / crop.width;
    const pictureHeight = picture.width / aspect;
    const image = {
      width: base.image.width * scale,
      x: picture.x - picture.width / 2 + (base.image.x - crop.x) * scale,
      y: picture.y - pictureHeight / 2 + (base.image.y - crop.y) * scale,
    };
    const cameraFraming =
      clip.camera ?? (sceneNeedsCamera(clip) ? WHOLE_FRAMING : null);
    // A camera filling the canvas is cut by the canvas's own corners, not
    // rounded again inside them.
    const cameraRadius =
      !clip.boxes && clip.preset === "camera-only" ? 0 : base.radius.camera;
    return {
      camera:
        camera && cameraFraming
          ? placedPicture(cameraFraming, frame, camera.aspect)
          : base.camera,
      cameraFront: clip.boxes?.cameraBehind ? 0 : 1,
      frame,
      image,
      opacity: {
        camera: shown.camera ? 1 : 0,
        screen: shown.screen ? 1 : 0,
      },
      radius: {
        camera: clip.radius?.camera ?? cameraRadius,
        screen: clip.radius?.screen ?? base.radius.screen,
      },
      screen,
    };
  };
