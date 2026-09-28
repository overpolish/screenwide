// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DEFAULT_COMPRESSION,
  defaultCameraOverlay,
} from "./recording-export-settings";
import {
  cameraResolutionScales,
  resolutionScales,
  sourceScalePercent,
} from "./resolution";
import {
  CameraOverlaySettings,
  EditorArtifact,
  RecordingExportChoices,
  RememberedCameraOverlay,
} from "./types";

type Size = { height: number; width: number };
type RecordingArtifact = Extract<EditorArtifact, { kind: "recording" }>;

export type SeededExportChoices = {
  bakeCamera: boolean;
  cameraCompression: number;
  cameraOverlay: CameraOverlaySettings;
  cameraResolutionScalePercent: number;
  collapseAudio: boolean;
  compression: number;
  resolutionScalePercent: number;
};

const MAX_COMPRESSION = 4;
/** The export's own lower bound on overlay sizes, as a share of the canvas. */
const MIN_OVERLAY_SHARE = 0.03;

const clamp = (value: number, min: number, max: number) =>
  Math.min(Math.max(value, min), max);

/**
 * Carry a remembered overlay onto another screen canvas and camera.
 *
 * The camera and its crop frame move as one piece: the frame's centre keeps
 * its share of the canvas, and the piece scales by the smaller of the two
 * canvas ratios so it never grows past the edges on a narrower screen. The crop
 * is kept as a share of the camera image, so a camera with another aspect
 * still gets a crop that lies inside its picture. Returns null when the result
 * would be smaller than the export accepts.
 */
export const restoredCameraOverlay = ({
  camera,
  remembered,
  screen,
}: {
  camera: Size;
  remembered: RememberedCameraOverlay;
  screen: Size;
}): CameraOverlaySettings | null => {
  const { overlay } = remembered;
  const oldImageWidth = overlay.cameraWidth;
  const oldImageHeight =
    oldImageWidth * (remembered.cameraHeight / remembered.cameraWidth);
  const oldLeft = overlay.cameraX - oldImageWidth / 2;
  const oldTop = overlay.cameraY - oldImageHeight / 2;
  const cropX = clamp((overlay.frameX - oldLeft) / oldImageWidth, 0, 1);
  const cropY = clamp((overlay.frameY - oldTop) / oldImageHeight, 0, 1);
  const cropWidth = clamp(overlay.frameWidth / oldImageWidth, 0, 1 - cropX);
  const cropHeight = clamp(overlay.frameHeight / oldImageHeight, 0, 1 - cropY);

  const scale = Math.min(
    screen.width / remembered.screenWidth,
    screen.height / remembered.screenHeight,
  );
  const imageWidth = oldImageWidth * scale;
  const imageHeight = imageWidth * (camera.height / camera.width);
  const frameWidth = cropWidth * imageWidth;
  const frameHeight = cropHeight * imageHeight;
  const centreX =
    ((overlay.frameX + overlay.frameWidth / 2) / remembered.screenWidth) *
    screen.width;
  const centreY =
    ((overlay.frameY + overlay.frameHeight / 2) / remembered.screenHeight) *
    screen.height;
  const spareWidth = screen.width - frameWidth;
  const spareHeight = screen.height - frameHeight;
  const frameX = clamp(
    centreX - frameWidth / 2,
    Math.min(0, spareWidth),
    Math.max(0, spareWidth),
  );
  const frameY = clamp(
    centreY - frameHeight / 2,
    Math.min(0, spareHeight),
    Math.max(0, spareHeight),
  );
  const restored = {
    cameraWidth: imageWidth,
    cameraX: frameX - cropX * imageWidth + imageWidth / 2,
    cameraY: frameY - cropY * imageHeight + imageHeight / 2,
    frameHeight,
    frameWidth,
    frameX,
    frameY,
    radiusPercent: overlay.radiusPercent,
  };
  const valid =
    Object.values(restored).every(Number.isFinite) &&
    imageWidth >= screen.width * MIN_OVERLAY_SHARE &&
    frameWidth >= screen.width * MIN_OVERLAY_SHARE &&
    frameHeight >= screen.height * MIN_OVERLAY_SHARE;
  return valid ? restored : null;
};

/** The remembered share of "Original", where this recording offers it. */
const restoredResolutionScale = (
  recording: RecordingArtifact,
  ratio: number | null,
) => {
  const original = sourceScalePercent(recording);
  if (ratio === null) return original;
  const scales = resolutionScales(recording);
  return (
    scales.find((scale) => Math.abs(scale / scales[0] - ratio) < 0.001) ??
    original
  );
};

/**
 * The export choices a newly opened capture starts with: the last recording
 * export's where they still apply, and the defaults otherwise.
 */
export const seededExportChoices = ({
  artifact,
  choices,
  screen,
}: {
  artifact: EditorArtifact | null;
  choices: RecordingExportChoices;
  /** The screen output the overlay is placed in. */
  screen: Size;
}): SeededExportChoices => {
  const recording = artifact?.kind === "recording" ? artifact : null;
  const camera = recording?.camera ?? null;
  const compressionFor = (remembered: number | null) => {
    if (!recording?.canCompress) return 0;
    return remembered !== null &&
      Number.isInteger(remembered) &&
      remembered >= 0 &&
      remembered <= MAX_COMPRESSION
      ? remembered
      : DEFAULT_COMPRESSION;
  };
  const cameraOverlay =
    camera && choices.cameraOverlay
      ? restoredCameraOverlay({
          camera,
          remembered: choices.cameraOverlay,
          screen,
        })
      : null;
  const cameraResolution = choices.cameraResolutionScalePercent;
  return {
    bakeCamera:
      camera !== null &&
      recording?.primaryKind === "screen" &&
      (choices.bakeCamera ?? false),
    cameraCompression: compressionFor(choices.cameraCompression),
    cameraOverlay: cameraOverlay ?? defaultCameraOverlay(artifact),
    cameraResolutionScalePercent:
      cameraResolution !== null &&
      cameraResolutionScales.includes(cameraResolution)
        ? cameraResolution
        : 100,
    collapseAudio: choices.collapseAudio ?? false,
    compression: compressionFor(choices.compression),
    resolutionScalePercent: recording
      ? restoredResolutionScale(recording, choices.resolutionScaleRatio)
      : 100,
  };
};
