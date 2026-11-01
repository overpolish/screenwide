// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultCameraOverlay } from "./export/recording-export-settings";
import {
  isLayerAtDefaultPlacement,
  isOverlayAtDefaultPlacement,
  layerAtDefaultPlacement,
  overlayAtDefaultPlacement,
} from "./layer-placement";
import { insetScreenshotPadding } from "./screenshot/screenshot-inset";
import {
  defaultRecordingOutput,
  defaultScreenshotOutput,
  resetScreenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot/screenshot-output";

import type { EditorArtifact } from "./types";

const placement = (settings: ScreenshotOutputSettings) => ({
  cropHeight: settings.cropHeight,
  cropWidth: settings.cropWidth,
  cropX: settings.cropX,
  cropY: settings.cropY,
  imageWidth: settings.imageWidth,
  imageX: settings.imageX,
  imageY: settings.imageY,
});

/** `settings` moved by (`dx`, `dy`) and scaled by `scale` about its frame's
 * corner, as a drag on the picture moves and sizes it. */
const dragged = (
  settings: ScreenshotOutputSettings,
  { dx, dy, scale }: { dx: number; dy: number; scale: number },
): ScreenshotOutputSettings => ({
  ...settings,
  cropHeight: settings.cropHeight * scale,
  cropWidth: settings.cropWidth * scale,
  cropX: settings.cropX + dx,
  cropY: settings.cropY + dy,
  imageWidth: settings.imageWidth * scale,
  imageX: settings.cropX + dx + (settings.imageX - settings.cropX) * scale,
  imageY: settings.cropY + dy + (settings.imageY - settings.cropY) * scale,
});

const expectClose = (actual: object, expected: object) => {
  for (const [key, value] of Object.entries(expected))
    expect(actual[key as keyof typeof actual], key).toBeCloseTo(
      value as number,
      6,
    );
};

describe("a recording's screen", () => {
  const screen = { height: 1_080, width: 1_920 };
  const fresh = defaultRecordingOutput({ primary: screen }).primary;

  it("is at its default placement in a fresh project", () => {
    expect(isLayerAtDefaultPlacement(fresh)).toBe(true);
    expectClose(placement(layerAtDefaultPlacement(fresh)), placement(fresh));
  });

  it("goes back where a fresh project places it once moved and sized", () => {
    const moved = dragged(fresh, { dx: 200, dy: -80, scale: 0.6 });
    expect(isLayerAtDefaultPlacement(moved)).toBe(false);
    const reset = layerAtDefaultPlacement(moved);
    expectClose(placement(reset), placement(fresh));
    expect(isLayerAtDefaultPlacement(reset)).toBe(true);
  });

  it("keeps its crop, padding, corners, shadow and turn, the frame fitted to the canvas", () => {
    const cropped: ScreenshotOutputSettings = {
      ...fresh,
      cropHeight: 540,
      cropWidth: 540,
      cropX: 300,
      cropY: 100,
      dropShadow: false,
      radiusPercent: 12,
      rotation: { x: 10, y: 0, z: 5 },
      sourceCrop: { height: 0.5, width: 0.28125, x: 0.1, y: 0.2 },
    };
    const padded = insetScreenshotPadding(cropped, screen, 0.06);
    const moved = dragged(padded, { dx: -120, dy: 40, scale: 0.5 });
    const reset = layerAtDefaultPlacement(moved);
    // A square frame fills the canvas's height, in its middle.
    expectClose(placement(reset), {
      cropHeight: 1_080,
      cropWidth: 1_080,
      cropX: 420,
      cropY: 0,
    });
    // The picture behind the frame scales with it, so the crop shows the
    // same part of it.
    const scale = reset.cropWidth / moved.cropWidth;
    expect(reset.imageWidth).toBeCloseTo(moved.imageWidth * scale, 6);
    expect(reset.imageX - reset.cropX).toBeCloseTo(
      (moved.imageX - moved.cropX) * scale,
      6,
    );
    expect(reset).toMatchObject({
      dropShadow: false,
      radiusPercent: 12,
      rotation: { x: 10, y: 0, z: 5 },
      sourceCrop: padded.sourceCrop,
    });
    expect(reset.inset).toBe(padded.inset);
  });
});

describe("a screenshot's layer", () => {
  it("goes back to fit the canvas, as a fresh screenshot is laid out", () => {
    const canvas = defaultScreenshotOutput(1_600, 900);
    const item = { height: 1_200, width: 800 };
    const fresh = resetScreenshotLayout(canvas, item);
    expect(isLayerAtDefaultPlacement(fresh)).toBe(true);
    const moved = dragged(fresh, { dx: 300, dy: 50, scale: 1.4 });
    expectClose(placement(layerAtDefaultPlacement(moved)), placement(fresh));
  });
});

describe("a camera drawn into the screen's picture", () => {
  const artifact = {
    camera: { height: 720, width: 1_280 },
    height: 1_080,
    kind: "recording",
    width: 1_920,
  } as EditorArtifact;
  const fresh = defaultCameraOverlay(artifact);

  it("goes back to the corner a fresh project gives it", () => {
    expect(isOverlayAtDefaultPlacement(fresh, fresh)).toBe(true);
    const moved = {
      ...fresh,
      cameraWidth: fresh.cameraWidth * 2,
      cameraX: 400,
      cameraY: 500,
      frameHeight: fresh.frameHeight * 2,
      frameWidth: fresh.frameWidth * 2,
      frameX: 400 - fresh.frameWidth,
      frameY: 500 - fresh.frameHeight,
    };
    expect(isOverlayAtDefaultPlacement(moved, fresh)).toBe(false);
    const reset = overlayAtDefaultPlacement(moved, fresh);
    expectClose(reset, fresh);
  });

  it("keeps its crop, fitted in the fresh corner's window", () => {
    // A square window cut from the middle of the camera, then moved.
    const square = {
      ...fresh,
      cameraX: 300,
      cameraY: 300,
      frameHeight: fresh.frameHeight,
      frameWidth: fresh.frameHeight,
      frameX: 300 - fresh.frameHeight / 2,
      frameY: 300 - fresh.frameHeight / 2,
      radiusPercent: 50,
    };
    const reset = overlayAtDefaultPlacement(square, fresh);
    expect(reset.frameWidth).toBeCloseTo(fresh.frameHeight, 6);
    expect(reset.frameHeight).toBeCloseTo(fresh.frameHeight, 6);
    expect(reset.frameY).toBeCloseTo(fresh.frameY, 6);
    // Centred in the fresh window, the camera still behind the window's
    // middle at its own size.
    expect(reset.cameraX).toBeCloseTo(fresh.cameraX, 6);
    expect(reset.cameraY).toBeCloseTo(fresh.cameraY, 6);
    expect(reset.cameraWidth).toBeCloseTo(fresh.cameraWidth, 6);
    expect(reset.radiusPercent).toBe(50);
  });

  it("is placed for the canvas it is drawn on now", () => {
    const wide = defaultCameraOverlay(artifact, {
      height: 1_080,
      width: 3_840,
    });
    expect(wide.frameWidth).toBeCloseTo(fresh.frameWidth * 2, 6);
    expect(isOverlayAtDefaultPlacement(fresh, wide)).toBe(false);
  });
});
