// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Measure, points } from "../../layer-measure";
import { UPRIGHT_ROTATION } from "../../layer-rotation";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings } from "../../types";

import {
  type Face,
  cameraFramingOf,
  drawnCameraFrame,
  followingFace,
  placedPicture,
} from "./recording-scene-framing";
import { SceneRect, recordingScenePanes } from "./recording-scene-geometry";
import {
  RecordingSceneClip,
  SceneBox,
  SceneFraming,
  SceneRotation,
  WHOLE_FRAMING,
  sceneNeedsCamera,
} from "./recording-scenes";

export type Picture = { width: number; x: number; y: number };

/** Where the panes are, in the canvas's output pixels: the screen's box and
 * the whole screen image behind it by its corner and width, then the
 * camera's box and the camera picture behind it by its centre and width;
 * the screen's corners as a length on its picture and the camera's in
 * percent of its box's shorter side; how opaque each pane is drawn, zero
 * for a pane the scene hides; how far the camera is drawn in front of the
 * screen, one in front and zero behind; how the screen is turned in depth
 * about its box's middle; and how far the scene pads the screen's box
 * outwards, as a length on its picture. The twin of `Placement` in
 * `src-tauri/crates/scenes/src/placement.rs`, which holds the screen's
 * corners and padding in canvas pixels; here they stay lengths until the
 * arrangement draws them at the picture's scale, which the selection alone
 * reads. */
export type Placement = {
  camera: Picture;
  cameraFront: number;
  frame: SceneRect;
  image: Picture;
  inset: Measure;
  opacity: { camera: number; screen: number };
  radius: { camera: number; screen: Measure };
  rotation: SceneRotation;
  screen: SceneRect;
};

/** The camera a scene can place: its picture's aspect, its overlay outside
 * every scene, and where the face is over it at the playhead, which a
 * camera that follows the face is placed by; none before it is found. */
export type SceneCamera = {
  aspect: number;
  overlay: CameraOverlaySettings;
  face?: Face | null;
};

type Canvas = { height: number; width: number };

/** The zoom the recording's own crop gives the camera. */
const ownZoom = (camera: SceneCamera) =>
  cameraFramingOf(camera.overlay, camera.aspect).zoom;

/** Where the recording's own composition draws the camera's picture: where
 * its overlay puts it, or, following the face, at the crop's zoom with the
 * face in the middle of its box. The twin of `drawn_camera` in
 * `Base::of`. */
export const ownCameraPicture = (camera: SceneCamera): Picture => {
  const { aspect, face, overlay } = camera;
  if (overlay.followFace === true && face)
    return placedPicture(
      followingFace({ focusX: 0.5, focusY: 0.5, zoom: ownZoom(camera) }, face),
      drawnCameraFrame(overlay, aspect),
      aspect,
    );
  return { width: overlay.cameraWidth, x: overlay.cameraX, y: overlay.cameraY };
};

/** Where the recording's own composition puts the panes, outside every
 * scene. */
export const basePlacement = (
  output: ScreenshotOutputSettings,
  camera: SceneCamera | null,
): Placement => ({
  camera: camera ? ownCameraPicture(camera) : { width: 0, x: 0, y: 0 },
  cameraFront: 1,
  // Where the camera is drawn rather than where its box is stored, so a
  // scene arriving starts from the camera on screen.
  frame: camera
    ? drawnCameraFrame(camera.overlay, camera.aspect)
    : { height: 0, width: 0, x: 0, y: 0 },
  image: { width: output.imageWidth, x: output.imageX, y: output.imageY },
  // The recording's own inset is already its crop's padding.
  inset: points(0),
  opacity: { camera: 1, screen: 1 },
  radius: {
    camera: camera?.overlay.radiusPercent ?? 0,
    screen: output.radius,
  },
  // A scene arrives from the screen's own turn and leaves for it.
  rotation: output.rotation ?? UPRIGHT_ROTATION,
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
 * unseen; a custom scene shows both, unless its boxes hide one.
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
      shown: {
        camera: boxes.cameraHidden !== true,
        screen: boxes.screenHidden !== true,
      },
    };
  const shapes = clip.shapes ?? {};
  // The composition's own places, each pane cropped to a shape of the
  // scene's own fitted inside its place, about its middle.
  if (clip.preset === "full" || clip.preset === "screen-only")
    return {
      camera:
        clip.preset === "full" && camera && shapes.camera
          ? fitted(base.frame, shapes.camera)
          : null,
      screen: shapes.screen ? fitted(base.screen, shapes.screen) : base.screen,
      shown: { camera: clip.preset === "full", screen: true },
    };
  // Without a camera no preset that places one plays, so any shape will do;
  // `arrange.rs` takes the same. Each pane is laid out at the shape the
  // scene crops it to, else its picture's.
  const panes = recordingScenePanes(clip.preset, canvas, {
    cameraAspect: shapes.camera ?? camera?.aspect ?? 16 / 9,
    screenAspect: shapes.screen ?? base.screen.width / base.screen.height,
    variant: clip.variant,
  });
  return {
    camera: panes.camera,
    screen: panes.screen ?? base.screen,
    shown: { camera: panes.camera !== null, screen: panes.screen !== null },
  };
};

/** The largest rect of `shape`, width over height, inside `place`, about
 * its middle. The twin of `fitted` in `target.rs`. */
const fitted = (place: SceneRect, shape: number): SceneRect => {
  const width = Math.min(place.width, place.height * shape);
  const height = width / shape;
  return {
    height,
    width,
    x: place.x + (place.width - width) / 2,
    y: place.y + (place.height - height) / 2,
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
    if (boxes.camera && boxes.shown.camera && !camera) return null;
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
    // A camera following the face is framed about it: its framing's focus
    // says where the face sits in the box.
    const recordingFollows = camera?.overlay.followFace === true;
    const follows = clip.followFace ?? recordingFollows;
    const face = follows ? camera?.face : null;
    const placed = (framing: SceneFraming) =>
      camera
        ? placedPicture(
            face ? followingFace(framing, face) : framing,
            frame,
            camera.aspect,
          )
        : base.camera;
    // A camera filling the canvas is cut by the canvas's own corners, not
    // rounded again inside them.
    const cameraRadius =
      !clip.boxes && clip.preset === "camera-only" ? 0 : base.radius.camera;
    return {
      // A full scene keeps the recording's own crop, followed or still as
      // the scene says.
      camera: !camera
        ? base.camera
        : clip.camera
          ? placed(clip.camera)
          : sceneNeedsCamera(clip)
            ? placed(WHOLE_FRAMING)
            : follows && !recordingFollows
              ? placed({ focusX: 0.5, focusY: 0.5, zoom: ownZoom(camera) })
              : !follows && recordingFollows
                ? ownCameraPicture({ ...camera, face: null })
                : base.camera,
      cameraFront: clip.boxes?.cameraBehind ? 0 : 1,
      frame,
      image,
      // The scene pads its box outwards, as the recording's inset pads its
      // crop.
      inset: clip.inset ?? points(0),
      opacity: {
        camera: shown.camera ? 1 : 0,
        screen: shown.screen ? 1 : 0,
      },
      radius: {
        camera: clip.radius?.camera ?? cameraRadius,
        screen: clip.radius?.screen ?? base.radius.screen,
      },
      rotation: clip.rotation ?? UPRIGHT_ROTATION,
      screen,
    };
  };
