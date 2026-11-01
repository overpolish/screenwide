// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import {
  defaultRecordingOutput,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import {
  CameraOverlaySettings,
  EditorArtifact,
  RecordingVideoTrackId,
} from "../../types";
import { editorSelectionTarget } from "../selection/selection-target";

const recording: EditorArtifact = {
  audioTracks: [],
  camera: {
    durationMs: 1_000,
    height: 720,
    originalSizeBytes: 1,
    path: "camera.mp4",
    width: 1_280,
  },
  canCompress: true,
  cursorDataVersion: null,
  durationMs: 1_000,
  extension: "mp4",
  hasCursorData: false,
  hasKeyboardData: false,
  height: 1_080,
  id: 2,
  keyboardDataVersion: null,
  kind: "recording",
  originalSizeBytes: 1,
  path: "recording.mp4",
  primaryKind: "screen",
  sourceScalePercent: 100,
  suggestedFileStem: "recording",
  width: 1_920,
};

const cameraTargetFor = ({
  bakeCamera = true,
  cameraOverlay = defaultCameraOverlay(recording),
  composedVideoTracks = ["primary", "camera"] as RecordingVideoTrackId[],
}: {
  bakeCamera?: boolean;
  cameraOverlay?: CameraOverlaySettings;
  composedVideoTracks?: RecordingVideoTrackId[];
} = {}) => {
  const overlay = vi.fn<(next: CameraOverlaySettings) => void>();
  const output =
    vi.fn<
      (track: RecordingVideoTrackId, next: ScreenshotOutputSettings) => void
    >();
  const target = editorSelectionTarget({
    artifact: recording,
    bakeCamera,
    cameraOverlay,
    composedVideoTracks,
    onCameraOverlayChange: overlay,
    onRecordingOutputChange: output,
    recordingOutput: defaultRecordingOutput({
      camera: { height: 720, width: 1_280 },
      primary: { height: 1_080, width: 1_920 },
    }),
    screenshotOutput: null,
    selectedScreenshotItemId: null,
    selectedTrack: "camera",
  });
  if (!target) throw new Error("Expected a camera selection target");
  return { output, overlay, target };
};

describe("baked camera selection", () => {
  it("shows the overlay's corners", () => {
    const overlay = defaultCameraOverlay(recording);
    const { target } = cameraTargetFor();

    expect(target.selection).toMatchObject({
      kind: "camera",
      label: "Camera",
      radius: overlay.radiusPercent,
    });
  });

  it("rounds the overlay's corners and shadows the camera track's output", () => {
    const { output, overlay, target } = cameraTargetFor();

    target.applyRadius(20);
    target.applyDropShadow(true);

    expect(overlay.mock.calls[0][0].radiusPercent).toBe(20);
    expect(output.mock.calls[0][0]).toBe("camera");
    expect(output.mock.calls[0][1].dropShadow).toBe(true);
  });
});
