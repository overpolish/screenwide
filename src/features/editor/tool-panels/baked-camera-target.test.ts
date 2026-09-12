// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { defaultCameraOverlay } from "../recording-export-settings";
import {
  defaultRecordingOutput,
  ScreenshotOutputSettings,
} from "../screenshot-output";
import {
  CameraOverlaySettings,
  EditorArtifact,
  RecordingVideoTrackId,
} from "../types";

import {
  editorSelectionTarget,
  selectionPanelHandlers,
} from "./selection-target";

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
  enabledVideoTracks = ["primary", "camera"] as RecordingVideoTrackId[],
}: {
  bakeCamera?: boolean;
  cameraOverlay?: CameraOverlaySettings;
  enabledVideoTracks?: RecordingVideoTrackId[];
} = {}) => {
  const bake = vi.fn<(bake: boolean) => void>();
  const overlay = vi.fn<(next: CameraOverlaySettings) => void>();
  const output =
    vi.fn<
      (track: RecordingVideoTrackId, next: ScreenshotOutputSettings) => void
    >();
  const target = editorSelectionTarget({
    artifact: recording,
    bakeCamera,
    cameraOverlay,
    enabledVideoTracks,
    onBakeCameraChange: bake,
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
  return { bake, output, overlay, target };
};

describe("baked camera selection", () => {
  it("shows the overlay frame, the camera's own source, and that it is baked", () => {
    const overlay = defaultCameraOverlay(recording);
    const { target } = cameraTargetFor();

    expect(target.selection).toMatchObject({
      canBake: true,
      height: Math.round(overlay.frameHeight),
      isBaked: true,
      kind: "camera",
      label: "Camera",
      radius: overlay.radiusPercent,
      sourceHeight: 720,
      sourceWidth: 1_280,
      width: Math.round(overlay.frameWidth),
    });
  });

  it("resizes the overlay about the frame's centre", () => {
    const settings = defaultCameraOverlay(recording);
    const { overlay, target } = cameraTargetFor({ cameraOverlay: settings });

    target.applyPlacement({ width: settings.frameWidth / 2 });

    const next = overlay.mock.calls[0][0];
    expect(next.frameWidth).toBeCloseTo(settings.frameWidth / 2);
    expect(next.frameHeight).toBeCloseTo(settings.frameHeight / 2);
    expect(next.frameX + next.frameWidth / 2).toBeCloseTo(
      settings.frameX + settings.frameWidth / 2,
    );
    expect(next.frameY + next.frameHeight / 2).toBeCloseTo(
      settings.frameY + settings.frameHeight / 2,
    );
    // The camera picture is carried by the same scale, so the crop window
    // keeps showing the same part of it.
    expect(next.cameraWidth).toBeCloseTo(settings.cameraWidth / 2);
  });

  it("puts the overlay back where a recording opens it", () => {
    const moved = {
      ...defaultCameraOverlay(recording),
      frameX: 12,
      frameY: 34,
    };
    const { overlay, target } = cameraTargetFor({ cameraOverlay: moved });

    target.reset();

    expect(overlay).toHaveBeenCalledWith(defaultCameraOverlay(recording));
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

describe("the bake switch", () => {
  it("asks the editor to bake, from either state", () => {
    const baked = cameraTargetFor();
    selectionPanelHandlers(baked.target).onBakeCameraChange?.(false);
    expect(baked.bake).toHaveBeenCalledWith(false);

    const split = cameraTargetFor({ bakeCamera: false });
    expect(split.target.selection.isBaked).toBe(false);
    selectionPanelHandlers(split.target).onBakeCameraChange?.(true);
    expect(split.bake).toHaveBeenCalledWith(true);
  });

  it("is out of reach while either track is left out", () => {
    const { target } = cameraTargetFor({
      bakeCamera: false,
      enabledVideoTracks: ["camera"],
    });

    expect(target.selection.canBake).toBe(false);
  });
});
