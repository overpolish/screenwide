// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { mappedAnnotationShape } from "../../annotations/annotation-mapping";
import {
  arrangedGroup,
  Arrangement,
  availableGroupArrangements,
} from "../../annotations/annotation-order";
import { AnnotationPoint } from "../../annotations/annotations";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings } from "../../types";

import {
  RecordingAnnotationClip,
  recordingAnnotationClipsMeet,
} from "./recording-annotations";

/** How far a camera annotation's weight follows the camera down; the twin of
 * `SMALLEST_WEIGHT` in `src-tauri/src/editor/annotations/camera_baked.rs`. */
const SMALLEST_WEIGHT = 0.5;

/** The kinds that only mark the picture, which can move between the screen
 * and the camera; an effect reads its own picture's frames. */
const MARKS = new Set(["arrow", "counter", "draw", "shape", "text"]);

/** Where the screen's and the camera's pixels fall on the canvas, as One
 * video draws them at the playhead: each picture's top-left corner and how
 * many canvas pixels one of its own pixels spans, and how many canvas pixels
 * one point of a screen annotation's size spans. */
export type RecordingCameraPlacement = {
  camera: { scale: number; x: number; y: number };
  screen: { scale: number; x: number; y: number };
  screenCaptureScale: number;
};

/** The placement One video draws the camera at, or null where the camera is
 * not drawn into the screen's video. `screen` is the screen's output and
 * `cameraOverlay` the camera's placement on it, both as the playhead's scene
 * arranges them. */
export const recordingCameraPlacement = ({
  cameraOverlay,
  cameraSource,
  screen,
  screenCaptureScale,
  screenSource,
}: {
  cameraOverlay: CameraOverlaySettings;
  cameraSource: { height: number; width: number } | undefined;
  screen: ScreenshotOutputSettings;
  screenCaptureScale: number;
  screenSource: { height: number; width: number } | undefined;
}): RecordingCameraPlacement | null => {
  if (!cameraSource || !screenSource) return null;
  const cameraScale = cameraOverlay.cameraWidth / cameraSource.width;
  const screenScale = screen.imageWidth / screenSource.width;
  if (
    !(cameraScale > 0 && Number.isFinite(cameraScale)) ||
    !(screenScale > 0 && Number.isFinite(screenScale))
  )
    return null;
  const cameraHeight =
    (cameraOverlay.cameraWidth * cameraSource.height) / cameraSource.width;
  return {
    camera: {
      scale: cameraScale,
      x: cameraOverlay.cameraX - cameraOverlay.cameraWidth / 2,
      y: cameraOverlay.cameraY - cameraHeight / 2,
    },
    screen: { scale: screenScale, x: screen.imageX, y: screen.imageY },
    screenCaptureScale: screenCaptureScale > 0 ? screenCaptureScale : 1,
  };
};

// Clips stack among those on their own track: each is drawn with its own
// picture.
const meet = (a: RecordingAnnotationClip, b: RecordingAnnotationClip) =>
  a.trackId === b.trackId && recordingAnnotationClipsMeet(a, b);

/** The track the group is handed to by a step past the end of its own, or
 * null where it cannot be: it spans both tracks, holds an effect or a pinned
 * clip, or the camera is not drawn into the video. */
const handoverTarget = (
  clips: RecordingAnnotationClip[],
  isMember: (clip: RecordingAnnotationClip) => boolean,
  placement: RecordingCameraPlacement | null,
): RecordingAnnotationClip["trackId"] | null => {
  const members = clips.filter(isMember);
  if (!placement || members.length === 0) return null;
  const track = members[0].trackId;
  if (
    members.some(
      (clip) =>
        clip.trackId !== track ||
        clip.pin !== undefined ||
        !MARKS.has(clip.annotation.shape.kind),
    )
  )
    return null;
  return track === "primary" ? "camera" : "primary";
};

/**
 * The moves the menu offers the group `isMember` picks. The camera is drawn
 * over the screen, so a step forward from the top of the screen's annotations
 * hands them to the camera and a step backward from the bottom of the
 * camera's hands them back. Moving to the front or back stays on the track.
 */
export const recordingAnnotationArrangements = (
  clips: RecordingAnnotationClip[],
  isMember: (clip: RecordingAnnotationClip) => boolean,
  placement: RecordingCameraPlacement | null,
) => {
  const own = availableGroupArrangements(clips, isMember, meet);
  const target = handoverTarget(clips, isMember, placement);
  return {
    canBringForward: own.canBringForward || target === "camera",
    canMoveToBack: own.canSendBackward,
    canMoveToFront: own.canBringForward,
    canSendBackward: own.canSendBackward || target === "primary",
  };
};

/**
 * `clips` with the group `isMember` picks moved through the stacking: within
 * its track, or past the end of it into the other track, where it keeps its
 * place and its look on the canvas at the playhead. Handed to the camera it
 * goes under the camera's own annotations; handed back, over the screen's.
 * The list is returned unchanged where nothing moves.
 */
export const arrangedRecordingAnnotations = ({
  arrangement,
  clips,
  isMember,
  placement,
}: {
  arrangement: Arrangement;
  clips: RecordingAnnotationClip[];
  isMember: (clip: RecordingAnnotationClip) => boolean;
  placement: RecordingCameraPlacement | null;
}): RecordingAnnotationClip[] => {
  const within = arrangedGroup(clips, isMember, { arrangement, meets: meet });
  if (within !== clips) return within;
  const target = handoverTarget(clips, isMember, placement);
  if (
    !placement ||
    !(
      (arrangement === "forward" && target === "camera") ||
      (arrangement === "backward" && target === "primary")
    )
  )
    return clips;
  const { camera, screen, screenCaptureScale } = placement;
  const toCamera = target === "camera";
  const [from, to] = toCamera ? [screen, camera] : [camera, screen];
  const carry = (point: AnnotationPoint) => ({
    x: (from.x + point.x * from.scale - to.x) / to.scale,
    y: (from.y + point.y * from.scale - to.y) / to.scale,
  });
  // A screen annotation's point is its capture's; a camera annotation's is a
  // camera pixel, drawn as far down as the camera is shown, to half.
  const cameraWeight = Math.min(1, Math.max(SMALLEST_WEIGHT, camera.scale));
  const weight = toCamera
    ? screenCaptureScale / cameraWeight
    : cameraWeight / screenCaptureScale;
  const carried = clips.filter(isMember).map((clip) => ({
    ...clip,
    annotation: {
      ...clip.annotation,
      shape: mappedAnnotationShape(clip.annotation.shape, carry),
      style: {
        ...clip.annotation.style,
        width: clip.annotation.style.width * weight,
      },
    },
    trackId: target,
  }));
  const rest = clips.filter((clip) => !isMember(clip));
  return toCamera ? [...carried, ...rest] : [...rest, ...carried];
};
