// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  restoredCameraOverlay,
  seededExportChoices,
} from "./recording-export-choices";
import { initialEditorSnapshot, EditorArtifact } from "./types";

const recording = (
  overrides: Partial<Extract<EditorArtifact, { kind: "recording" }>> = {},
): EditorArtifact => ({
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
  id: 1,
  keyboardDataVersion: null,
  kind: "recording",
  originalSizeBytes: 1,
  path: "recording.mp4",
  primaryKind: "screen",
  sourceScalePercent: 100,
  suggestedFileStem: "recording",
  width: 1_920,
  ...overrides,
});

const noChoices = initialEditorSnapshot("recording").recordingExportChoices;

/** A camera in the bottom-left corner, cropped to its middle half. */
const bottomLeft = {
  cameraHeight: 720,
  cameraWidth: 1_280,
  overlay: {
    cameraWidth: 640,
    cameraX: 400,
    cameraY: 800,
    frameHeight: 180,
    frameWidth: 320,
    frameX: 240,
    frameY: 710,
    radiusPercent: 12,
  },
  screenHeight: 1_080,
  screenWidth: 1_920,
};

describe("restoredCameraOverlay", () => {
  it("returns the remembered overlay unchanged on the same geometry", () => {
    expect(
      restoredCameraOverlay({
        camera: { height: 720, width: 1_280 },
        remembered: bottomLeft,
        screen: { height: 1_080, width: 1_920 },
      }),
    ).toEqual(bottomLeft.overlay);
  });

  it("keeps the corner and the crop on a larger screen and a squarer camera", () => {
    const screen = { height: 2_160, width: 3_840 };
    const restored = restoredCameraOverlay({
      camera: { height: 1_080, width: 1_440 },
      remembered: bottomLeft,
      screen,
    });
    if (!restored) throw new Error("expected an overlay");

    const imageHeight = restored.cameraWidth * (1_080 / 1_440);
    const imageLeft = restored.cameraX - restored.cameraWidth / 2;
    const imageTop = restored.cameraY - imageHeight / 2;
    // The crop is still the middle half of the new camera's picture.
    expect(restored.frameWidth / restored.cameraWidth).toBeCloseTo(0.5);
    expect(restored.frameHeight / imageHeight).toBeCloseTo(0.5);
    expect((restored.frameX - imageLeft) / restored.cameraWidth).toBeCloseTo(
      0.25,
    );
    expect((restored.frameY - imageTop) / imageHeight).toBeCloseTo(0.25);
    // Its centre keeps the same share of the canvas.
    expect(
      (restored.frameX + restored.frameWidth / 2) / screen.width,
    ).toBeCloseTo((240 + 160) / 1_920);
    expect(restored.radiusPercent).toBe(12);
  });

  it("stays inside a much narrower screen", () => {
    const screen = { height: 1_080, width: 600 };
    const restored = restoredCameraOverlay({
      camera: { height: 720, width: 1_280 },
      remembered: bottomLeft,
      screen,
    });
    if (!restored) throw new Error("expected an overlay");
    expect(restored.frameX).toBeGreaterThanOrEqual(0);
    expect(restored.frameX + restored.frameWidth).toBeLessThanOrEqual(
      screen.width,
    );
    expect(restored.frameY + restored.frameHeight).toBeLessThanOrEqual(
      screen.height,
    );
  });
});

describe("seededExportChoices", () => {
  const screen = { height: 1_080, width: 1_920 };

  it("starts from the defaults when nothing has been exported", () => {
    const seeded = seededExportChoices({
      artifact: recording({ sourceScalePercent: 200 }),
      choices: noChoices,
      screen,
    });
    expect(seeded).toMatchObject({
      bakeCamera: false,
      cameraCompression: 2,
      cameraResolutionScalePercent: 100,
      collapseAudio: false,
      compression: 2,
      resolutionScalePercent: 200,
    });
  });

  it("maps the remembered scale by its label, not its percentage", () => {
    const scaleFor = (sourceScalePercent: number, ratio: number) =>
      seededExportChoices({
        artifact: recording({ sourceScalePercent }),
        choices: { ...noChoices, resolutionScaleRatio: ratio },
        screen,
      }).resolutionScalePercent;
    // "Original" chosen on a 1x capture is still "Original" on a Retina one.
    expect(scaleFor(200, 1)).toBe(200);
    expect(scaleFor(200, 0.5)).toBe(100);
    // Half of a 150% capture is not offered, so it falls back to Original.
    expect(scaleFor(150, 0.5)).toBe(150);
  });

  it("bakes only a recording that has a screen and a camera", () => {
    const choices = { ...noChoices, bakeCamera: true };
    expect(
      seededExportChoices({ artifact: recording(), choices, screen })
        .bakeCamera,
    ).toBe(true);
    expect(
      seededExportChoices({
        artifact: recording({ camera: null }),
        choices,
        screen,
      }).bakeCamera,
    ).toBe(false);
  });

  it("keeps an uncompressible recording at zero compression", () => {
    const seeded = seededExportChoices({
      artifact: recording({ canCompress: false }),
      choices: { ...noChoices, cameraCompression: 3, compression: 3 },
      screen,
    });
    expect(seeded.compression).toBe(0);
    expect(seeded.cameraCompression).toBe(0);
  });
});
