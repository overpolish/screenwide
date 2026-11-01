// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import {
  defaultScreenshotOutput,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../../screenshot/screenshot-output";
import { CameraOverlaySettings, EditorArtifact } from "../../types";

import { editorFrameTarget } from "./frame-target";

const screenshotArtifact: EditorArtifact = {
  extension: "png",
  height: 900,
  id: 1,
  items: [{ annotations: [], height: 900, id: 7, width: 1_600 }],
  kind: "screenshot",
  suggestedFileStem: "capture",
  width: 1_600,
};

const recordingArtifact = {
  extension: "mp4",
  height: 900,
  id: 2,
  kind: "recording",
  width: 1_600,
} as EditorArtifact;

describe("frame radius target", () => {
  it("changes the screenshot canvas radius while preserving the layer radius", () => {
    const output = {
      ...defaultScreenshotOutput(1_600, 900, {
        background: 4,
        screenshot: 18,
      }),
      items: [
        {
          id: 7,
          output: defaultScreenshotOutput(800, 450, { screenshot: 27 }),
        },
      ],
    };
    const apply = vi.fn<(next: ScreenshotWorkspaceOutputSettings) => void>();
    const target = editorFrameTarget({
      artifact: screenshotArtifact,
      bakeCamera: false,
      cameraOverlay: defaultCameraOverlay(),
      composedVideoTracks: [],
      onCanvasResize: apply,
      recordingOutput: null,
      screenshotOutput: output,
      selectedScreenshotItemId: 7,
      selectedTrack: null,
    });

    target?.applyRadius(11);

    expect(target?.snapshot.radius).toBe(4);
    expect(apply.mock.calls[0][0].backgroundRadiusPercent).toBe(11);
    expect(apply.mock.calls[0][0].items[0].output.radiusPercent).toBe(27);
  });

  it("changes the selected recording canvas radius", () => {
    const output = defaultScreenshotOutput(1_600, 900, {
      background: 3,
      screenshot: 21,
    });
    const apply = vi.fn<(next: ScreenshotOutputSettings) => void>();
    const target = editorFrameTarget({
      artifact: recordingArtifact,
      bakeCamera: false,
      cameraOverlay: defaultCameraOverlay(),
      composedVideoTracks: ["primary"],
      onRecordingOutputChange: (_track, next) => {
        apply(next);
      },
      recordingOutput: { camera: output, primary: output },
      screenshotOutput: null,
      selectedScreenshotItemId: null,
      selectedTrack: "primary",
    });

    target?.applyRadius(9);

    expect(target?.snapshot.radius).toBe(3);
    expect(apply.mock.calls[0][0].backgroundRadiusPercent).toBe(9);
    expect(apply.mock.calls[0][0].radiusPercent).toBe(21);
  });
});

describe("the frame's dimensions", () => {
  /** A 400 by 300 layer whose frame's middle stands at (500, 300) on a 1600
   * by 900 canvas. */
  const layer = (): ScreenshotOutputSettings => ({
    ...defaultScreenshotOutput(1_600, 900),
    cropHeight: 300,
    cropWidth: 400,
    cropX: 300,
    cropY: 150,
    imageWidth: 400,
    imageX: 300,
    imageY: 150,
  });

  it("scale a screenshot's layers with the canvas, in one commit", () => {
    const apply = vi.fn<(next: ScreenshotWorkspaceOutputSettings) => void>();
    const target = editorFrameTarget({
      artifact: screenshotArtifact,
      bakeCamera: false,
      cameraOverlay: defaultCameraOverlay(),
      composedVideoTracks: [],
      onCanvasResize: apply,
      recordingOutput: null,
      screenshotOutput: {
        ...defaultScreenshotOutput(1_600, 900),
        items: [{ id: 7, output: layer() }],
      },
      selectedScreenshotItemId: 7,
      selectedTrack: null,
    });

    target?.apply({ height: 450, width: 800 });

    expect(apply).toHaveBeenCalledOnce();
    const next = apply.mock.calls[0][0];
    expect([next.width, next.height]).toEqual([800, 450]);
    const { output } = next.items[0];
    expect([output.width, output.height]).toEqual([800, 450]);
    expect(output.cropWidth).toBeCloseTo(200);
    expect(output.cropHeight).toBeCloseTo(150);
    expect(output.cropX + output.cropWidth / 2).toBeCloseTo(250);
    expect(output.cropY + output.cropHeight / 2).toBeCloseTo(150);
    expect(output.imageWidth).toBeCloseTo(200);
    expect(output.imageX).toBeCloseTo(output.cropX);
  });

  it("keep a layer's shape and place on a canvas narrowed in one axis", () => {
    const apply = vi.fn<(next: ScreenshotOutputSettings) => void>();
    const target = editorFrameTarget({
      artifact: recordingArtifact,
      bakeCamera: false,
      cameraOverlay: defaultCameraOverlay(),
      composedVideoTracks: ["primary"],
      onRecordingOutputChange: (_track, next) => {
        apply(next);
      },
      recordingOutput: { camera: layer(), primary: layer() },
      screenshotOutput: null,
      selectedScreenshotItemId: null,
      selectedTrack: "primary",
    });

    target?.apply({ height: 900, width: 800 });

    const next = apply.mock.calls[0][0];
    // Half as wide: the layer halves, keeping its shape, and its middle
    // stays at the same share of the canvas across and down.
    expect(next.cropWidth / next.cropHeight).toBeCloseTo(4 / 3);
    expect(next.cropWidth).toBeCloseTo(200);
    expect(next.cropX + next.cropWidth / 2).toBeCloseTo(250);
    expect(next.cropY + next.cropHeight / 2).toBeCloseTo(300);
  });

  it("scale a camera drawn into the screen's canvas with it", () => {
    const overlays = vi.fn<(next: CameraOverlaySettings) => void>();
    const outputs = vi.fn<(next: ScreenshotOutputSettings) => void>();
    const cameraOverlay = defaultCameraOverlay();
    const target = editorFrameTarget({
      artifact: recordingArtifact,
      bakeCamera: true,
      cameraOverlay,
      composedVideoTracks: ["primary", "camera"],
      onCameraOverlayChange: overlays,
      onRecordingOutputChange: (_track, next) => {
        outputs(next);
      },
      recordingOutput: { camera: layer(), primary: layer() },
      screenshotOutput: null,
      selectedScreenshotItemId: null,
      selectedTrack: "primary",
    });

    target?.apply({ height: 450, width: 800 });

    expect(outputs).toHaveBeenCalledOnce();
    const camera = overlays.mock.calls[0][0];
    expect(camera.frameWidth).toBeCloseTo(cameraOverlay.frameWidth / 2);
    expect(camera.cameraWidth).toBeCloseTo(cameraOverlay.cameraWidth / 2);
    expect(camera.frameX + camera.frameWidth / 2).toBeCloseTo(
      (cameraOverlay.frameX + cameraOverlay.frameWidth / 2) / 2,
    );
  });
});
